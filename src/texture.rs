use image::io::Reader;
use image::{ImageFormat, Pixel};

use std::fs::File;
use std::io::BufReader;
use std::ops::Index;
use std::path::Path;

use crate::math::{M4, V2, V3, V4};

pub trait Surface: Send + Sync {
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn get_f(&self, index: V2) -> V4;
}

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

        let normalize_component = |c| c as f32 / 255 as f32;

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

pub struct YCbCrTexture {
    luma: Texture,
    chroma: Texture,
    yuv_transform: M4,
}

impl YCbCrTexture {
    pub fn load_png<P: AsRef<Path>>(
        luma: P,
        chroma: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let luma = Texture::load_png(luma)?;
        let chroma = Texture::load_png(chroma)?;

        let kr = 0.299;
        let kg = 0.587;
        let kb = 0.114;

        let yuv_transform = M4::new(
            [1.0, 1.0, 1.0, 0.0],
            [0.0, -(kb / kg) * (2.0 - 2.0 * kb), (2.0 - 2.0 * kb), 0.0],
            [2.0 - 2.0 * kr, -(kr / kg) * (2.0 - 2.0 * kr), 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        );

        Ok(Self {
            luma,
            chroma,
            yuv_transform,
        })
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
        let luma = self.luma.get_f(index).x();
        let chroma = self.chroma.get_f(index);

        let yuv = V3::new(luma, chroma.x() - 0.5, chroma.y() - 0.5);

        let color = (self.yuv_transform * yuv)
            .min(V3::fill(1.0))
            .max(V3::fill(0.0))
            .powf(2.2);

        V4::new(color.x(), color.y(), color.z(), 1.0)
    }
}
