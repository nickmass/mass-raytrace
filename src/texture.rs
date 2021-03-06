use image::io::Reader;
use image::{ImageFormat, Pixel};

use std::fs::File;
use std::io::BufReader;
use std::ops::Index;
use std::path::Path;
use std::sync::Arc;

use crate::math::{M4, V2, V3, V4};

pub trait Surface: Send + Sync {
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn get_f(&self, index: V2) -> V4;
}

type SharedTexture = Arc<Texture>;

#[derive(Debug, Clone)]
pub struct Texture {
    width: u32,
    height: u32,
    pixels: Vec<V4>,
}

impl Texture {
    pub fn load_png<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();

        let file = BufReader::new(File::open(path)?);

        let image = Reader::with_format(file, ImageFormat::Png).decode()?;
        let image = image.to_rgba8();

        let width = image.width();
        let height = image.height();

        let normalize_component = |c| c as f32 / 255.0;

        let mut pixels = Vec::new();

        for p in image.pixels() {
            if let &[r, g, b, a] = p.channels() {
                let color = V4::new(
                    normalize_component(r),
                    normalize_component(g),
                    normalize_component(b),
                    normalize_component(a),
                );

                pixels.push(color);
            } else {
                unreachable!("expected 4 channel image")
            }
        }

        Ok(Texture {
            width,
            height,
            pixels,
        })
    }

    pub fn shared(self) -> SharedTexture {
        Arc::new(self)
    }
}

impl Index<(usize, usize)> for Texture {
    type Output = V4;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.pixels[index.1 * self.width() as usize + index.0]
    }
}

impl Surface for Texture {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn get_f(&self, index: V2) -> V4 {
        let x = index.x();
        let y = index.y();
        let x = if x < 0.0 { 1.0 - x.abs().fract() } else { x };
        let y = if y < 0.0 { 1.0 - y.abs().fract() } else { y };
        let x = if x == 1.0 { 1.0 } else { x.fract() };
        let y = if y == 1.0 { 1.0 } else { y.fract() };

        let x = x * (self.width() - 1) as f32;
        let y = y * (self.height() - 1) as f32;

        let x0 = x.floor() as usize;
        let x1 = x.ceil() as usize;

        let y0 = y.floor() as usize;
        let y1 = y.ceil() as usize;

        let t = x - x0 as f32;

        let p0 = self[(x0, y0)] * (1.0 - t) + self[(x1, y0)] * t;
        let p1 = self[(x0, y1)] * (1.0 - t) + self[(x1, y1)] * t;

        let t = y - y0 as f32;

        p1 * t + p0 * (1.0 - t)
    }
}

impl Surface for SharedTexture {
    fn width(&self) -> u32 {
        Texture::width(self)
    }

    fn height(&self) -> u32 {
        Texture::height(self)
    }

    fn get_f(&self, index: V2) -> V4 {
        Texture::get_f(self, index)
    }
}

const KR: f32 = 0.2126;
const KG: f32 = 0.7152;
const KB: f32 = 0.0722;

const YUV_TRANSFORM: M4 = M4::new(
    V4::new(1.0, 1.0, 1.0, 0.0),
    V4::new(0.0, -(KB / KG) * (2.0 - 2.0 * KB), 2.0 - 2.0 * KB, 0.0),
    V4::new(2.0 - 2.0 * KR, -(KR / KG) * (2.0 - 2.0 * KR), 0.0, 0.0),
    V4::new(0.0, 0.0, 0.0, 1.0),
);

pub struct YCbCrTexture {
    luma: Texture,
    chroma: Texture,
}

impl YCbCrTexture {
    pub fn load_png<P: AsRef<Path>>(
        luma: P,
        chroma: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let luma = Texture::load_png(luma)?;
        let chroma = Texture::load_png(chroma)?;

        Ok(Self { luma, chroma })
    }
}

impl Surface for YCbCrTexture {
    fn width(&self) -> u32 {
        self.luma.width()
    }

    fn height(&self) -> u32 {
        self.luma.height()
    }

    fn get_f(&self, index: V2) -> V4 {
        let luma = self.luma.get_f(index);
        let chroma = self.chroma.get_f(index);

        let yuv = V3::new(luma.x(), chroma.x() - 0.5, chroma.y() - 0.5);

        let color = YUV_TRANSFORM
            .transform_point(yuv)
            .min(V3::fill(1.0))
            .max(V3::fill(0.0))
            .powf(2.2);

        color.expand(1.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BlendMode {
    Lighten,
    Darken,
    Addition,
    Subtraction,
}

impl BlendMode {
    fn blend(&self, left: V4, right: V4) -> V4 {
        match self {
            BlendMode::Lighten => left.max(right),
            BlendMode::Darken => left.min(right),
            BlendMode::Addition => (left + right).min(V4::one()),
            BlendMode::Subtraction => (left - right).max(V4::zero()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextureBlend<L: Surface, R: Surface> {
    blend_mode: BlendMode,
    left: L,
    right: R,
}

impl<L: Surface, R: Surface> TextureBlend<L, R> {
    pub fn new(blend_mode: BlendMode, left: L, right: R) -> Self {
        Self {
            blend_mode,
            left,
            right,
        }
    }
}

impl<L: Surface, R: Surface> Surface for TextureBlend<L, R> {
    fn width(&self) -> u32 {
        self.left.width().max(self.right.width())
    }

    fn height(&self) -> u32 {
        self.left.height().max(self.right.height())
    }

    fn get_f(&self, index: V2) -> V4 {
        let l = self.left.get_f(index);
        let r = self.right.get_f(index);

        self.blend_mode.blend(l, r)
    }
}
