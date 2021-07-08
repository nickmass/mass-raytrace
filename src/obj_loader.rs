use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub trait ObjGroupFilter {
    fn include_group(&self, group_name: Option<&str>) -> bool;
}

impl ObjGroupFilter for () {
    fn include_group(&self, _group_name: Option<&str>) -> bool {
        true
    }
}

pub trait ObjBuilder {
    type Vertex: Copy;
    type Normal: Copy;
    type Texture: Copy;
    type Face;
    type Error: std::error::Error + 'static;
    fn include_group(&mut self, context: &ObjContext) -> bool {
        true
    }
    fn load_materials(&mut self, context: &ObjContext) {}
    fn build_vertex(&mut self, context: &ObjContext, x: f32, y: f32, z: f32) -> Self::Vertex;
    fn build_normal(&mut self, context: &ObjContext, x: f32, y: f32, z: f32) -> Self::Normal;
    fn build_uv(&mut self, context: &ObjContext, x: f32, y: f32) -> Self::Texture;
    fn build_face(
        &mut self,
        context: &ObjContext,
        face_a: (Self::Vertex, Self::Normal, Self::Texture),
        face_b: (Self::Vertex, Self::Normal, Self::Texture),
        face_c: (Self::Vertex, Self::Normal, Self::Texture),
    ) -> Result<Self::Face, Self::Error>;
}

pub fn obj_fns<V, N, UV, F, FV, FN, FUV, FF, OF>(
    vertex_fn: FV,
    normal_fn: FN,
    uv_fn: FUV,
    face_fn: FF,
) -> FnObjBuilder<V, N, UV, F, FV, FN, FUV, FF, OF>
where
    V: Copy,
    N: Copy,
    UV: Copy,
    FV: FnMut(f32, f32, f32) -> V,
    FN: FnMut(f32, f32, f32) -> N,
    FUV: FnMut(f32, f32) -> UV,
    FF: FnMut((V, N, UV), (V, N, UV), (V, N, UV)) -> F,
    OF: ObjGroupFilter,
{
    FnObjBuilder {
        vertex_fn,
        normal_fn,
        uv_fn,
        face_fn,
        group_filter: None,
        v_marker: PhantomData::default(),
        n_marker: PhantomData::default(),
        uv_marker: PhantomData::default(),
        f_marker: PhantomData::default(),
    }
}

pub struct FnObjBuilder<
    V: Copy,
    N: Copy,
    UV: Copy,
    F,
    FV: FnMut(f32, f32, f32) -> V,
    FN: FnMut(f32, f32, f32) -> N,
    FUV: FnMut(f32, f32) -> UV,
    FF: FnMut((V, N, UV), (V, N, UV), (V, N, UV)) -> F,
    OF: ObjGroupFilter,
> {
    vertex_fn: FV,
    normal_fn: FN,
    uv_fn: FUV,
    face_fn: FF,
    group_filter: Option<OF>,
    v_marker: PhantomData<V>,
    n_marker: PhantomData<N>,
    uv_marker: PhantomData<UV>,
    f_marker: PhantomData<F>,
}

impl<V, N, UV, F, FV, FN, FUV, FF, OF> FnObjBuilder<V, N, UV, F, FV, FN, FUV, FF, OF>
where
    V: Copy,
    N: Copy,
    UV: Copy,
    FV: FnMut(f32, f32, f32) -> V,
    FN: FnMut(f32, f32, f32) -> N,
    FUV: FnMut(f32, f32) -> UV,
    FF: FnMut((V, N, UV), (V, N, UV), (V, N, UV)) -> F,
    OF: ObjGroupFilter,
{
    pub fn with_filter(mut self, filter: OF) -> Self {
        self.group_filter = Some(filter);
        self
    }
}

impl<V, N, UV, F, FV, FN, FUV, FF, OF> ObjBuilder for FnObjBuilder<V, N, UV, F, FV, FN, FUV, FF, OF>
where
    V: Copy,
    N: Copy,
    UV: Copy,
    FV: FnMut(f32, f32, f32) -> V,
    FN: FnMut(f32, f32, f32) -> N,
    FUV: FnMut(f32, f32) -> UV,
    FF: FnMut((V, N, UV), (V, N, UV), (V, N, UV)) -> F,
    OF: ObjGroupFilter,
{
    type Vertex = V;
    type Normal = N;
    type Texture = UV;
    type Face = F;
    type Error = std::convert::Infallible;

    fn build_vertex(&mut self, _context: &ObjContext, x: f32, y: f32, z: f32) -> Self::Vertex {
        (self.vertex_fn)(x, y, z)
    }

    fn build_normal(&mut self, _context: &ObjContext, x: f32, y: f32, z: f32) -> Self::Normal {
        (self.normal_fn)(x, y, z)
    }

    fn build_uv(&mut self, _context: &ObjContext, x: f32, y: f32) -> Self::Texture {
        (self.uv_fn)(x, y)
    }

    fn build_face(
        &mut self,
        _context: &ObjContext,
        face_a: (Self::Vertex, Self::Normal, Self::Texture),
        face_b: (Self::Vertex, Self::Normal, Self::Texture),
        face_c: (Self::Vertex, Self::Normal, Self::Texture),
    ) -> Result<Self::Face, Self::Error> {
        Ok((self.face_fn)(face_a, face_b, face_c))
    }

    fn include_group(&mut self, context: &ObjContext) -> bool {
        if let Some(filter) = &self.group_filter {
            filter.include_group(context.group())
        } else {
            true
        }
    }
}

use crate::geom::Triangle;
use crate::material::{Lambertian, Metal};
use crate::math::{V2, V3};
use crate::texture::{SharedTexture, SolidColor, Surface, Texture, WrapMode};
pub struct SimpleTexturedBuilder {
    textures: HashMap<String, SharedTexture>,
    diffuse: HashMap<String, V3>,
    filtered_groups: HashSet<String>,
    wrapping: WrapMode,
}

impl SimpleTexturedBuilder {
    pub fn new(wrapping: WrapMode) -> Self {
        SimpleTexturedBuilder {
            textures: HashMap::new(),
            diffuse: HashMap::new(),
            filtered_groups: HashSet::new(),
            wrapping,
        }
    }

    pub fn with_filter<I, S>(wrapping: WrapMode, filtered_groups: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let filtered_groups = filtered_groups.into_iter().map(|s| s.into()).collect();
        SimpleTexturedBuilder {
            textures: HashMap::new(),
            diffuse: HashMap::new(),
            filtered_groups,
            wrapping,
        }
    }

    fn process_material_library(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = BufReader::new(File::open(path)?);
        let mut line = String::new();
        let mut current_material = None;
        loop {
            line.clear();
            let bytes = file.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }

            let parts: Vec<_> = line.trim().split_whitespace().collect();
            match parts.get(0).as_deref().copied() {
                Some("newmtl") => {
                    if let Some(name) = parts.get(1) {
                        current_material = Some(name.to_string());
                    }
                }
                Some("Kd") => {
                    if let Some(current_material) = current_material.as_ref() {
                        let x = parts.get(1).and_then(|n| n.parse::<f32>().ok());
                        let y = parts.get(2).and_then(|n| n.parse::<f32>().ok());
                        let z = parts.get(3).and_then(|n| n.parse::<f32>().ok());
                        if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                            let diffuse = V3::new(x, y, z);
                            self.diffuse.insert(current_material.clone(), diffuse);
                        }
                    }
                }
                Some("map_Kd") => {
                    if let (Some(texture_file), Some(current_material)) =
                        (parts.get(1), current_material.as_ref())
                    {
                        let texture_path = path.with_file_name(texture_file);
                        let texture = Texture::load_png(texture_path, self.wrapping)?.shared();
                        self.textures.insert(current_material.clone(), texture);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum SimpleTexturedBuilderError {
    NoMaterialForFace,
}

impl std::fmt::Display for SimpleTexturedBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No material found for face")
    }
}

impl std::error::Error for SimpleTexturedBuilderError {}

impl ObjBuilder for SimpleTexturedBuilder {
    type Vertex = V3;
    type Normal = V3;
    type Texture = V2;
    type Face = Triangle<Lambertian<Arc<dyn Surface>>>;
    type Error = SimpleTexturedBuilderError;

    fn load_materials(&mut self, context: &ObjContext) {
        if let Some(path) = context.material_library() {
            match self.process_material_library(path) {
                Err(e) => eprintln!("unable to load material library: {} {:?}", e, e),
                _ => (),
            }
        }
    }

    fn build_vertex(&mut self, _context: &ObjContext, x: f32, y: f32, z: f32) -> Self::Vertex {
        V3::new(x, y, z)
    }

    fn build_normal(&mut self, _context: &ObjContext, x: f32, y: f32, z: f32) -> Self::Normal {
        V3::new(x, y, z)
    }

    fn build_uv(&mut self, _context: &ObjContext, x: f32, y: f32) -> Self::Texture {
        V2::new(x, 1.0 - y)
    }

    fn build_face(
        &mut self,
        context: &ObjContext,
        face_a: (Self::Vertex, Self::Normal, Self::Texture),
        face_b: (Self::Vertex, Self::Normal, Self::Texture),
        face_c: (Self::Vertex, Self::Normal, Self::Texture),
    ) -> Result<Self::Face, Self::Error> {
        if let Some(texture) = context.material().and_then(|m| self.textures.get(m)) {
            let material: Lambertian<Arc<dyn Surface>> = Lambertian::new(texture.clone());
            Ok(Triangle::with_norms_and_uvs(
                material, face_a, face_b, face_c,
            ))
        } else if let Some(diffuse) = context.material().and_then(|m| self.diffuse.get(m)) {
            let color = SolidColor(diffuse.expand(1.0));
            let material: Lambertian<Arc<dyn Surface>> = Lambertian::new(Arc::new(color));
            Ok(Triangle::with_norms_and_uvs(
                material, face_a, face_b, face_c,
            ))
        } else {
            Err(SimpleTexturedBuilderError::NoMaterialForFace)
        }
    }

    fn include_group(&mut self, context: &ObjContext) -> bool {
        if let Some(group) = context.group().as_ref() {
            !self.filtered_groups.contains(*group)
        } else {
            true
        }
    }
}

#[derive(Default)]
pub struct ObjContext {
    group_name: Option<String>,
    material_name: Option<String>,
    material_library: Option<PathBuf>,
}

impl ObjContext {
    pub fn group(&self) -> Option<&str> {
        self.group_name.as_deref()
    }
    pub fn material(&self) -> Option<&str> {
        self.material_name.as_deref()
    }
    pub fn material_library(&self) -> Option<&Path> {
        self.material_library.as_deref()
    }
}

pub struct ObjLoader;

impl ObjLoader {
    pub fn load<P: AsRef<Path>, B: ObjBuilder>(
        path: P,
        mut builder: B,
    ) -> Result<Vec<B::Face>, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let mut file = BufReader::new(File::open(path)?);

        let mut vertexes = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut faces = Vec::new();

        let mut line = String::new();

        let mut context = ObjContext::default();

        let mut include_faces = builder.include_group(&context);
        loop {
            line.clear();
            let bytes = file.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }

            let parts: Vec<_> = line.split_whitespace().collect();

            match parts.get(0).map(|s| *s) {
                Some("v") => {
                    let x = parts.get(1).and_then(|n| n.parse().ok());
                    let y = parts.get(2).and_then(|n| n.parse().ok());
                    let z = parts.get(3).and_then(|n| n.parse().ok());

                    if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                        let vert = builder.build_vertex(&context, x, y, z);
                        vertexes.push(vert);
                    } else {
                        return Err(format!("unable to parse vertex: {}", line))?;
                    }
                }
                Some("vn") => {
                    let x = parts.get(1).and_then(|n| n.parse().ok());
                    let y = parts.get(2).and_then(|n| n.parse().ok());
                    let z = parts.get(3).and_then(|n| n.parse().ok());

                    if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                        let normal = builder.build_normal(&context, x, y, z);
                        normals.push(normal);
                    } else {
                        return Err(format!("unable to parse normal: {}", line))?;
                    }
                }
                Some("vt") => {
                    let u = parts.get(1).and_then(|n| n.parse().ok());
                    let v = parts.get(2).and_then(|n| n.parse().ok());

                    if let (Some(u), Some(v)) = (u, v) {
                        let uv = builder.build_uv(&context, u, v);
                        uvs.push(uv);
                    } else {
                        return Err(format!("unable to parse texture coord: {}", line))?;
                    }
                }
                Some("f") => {
                    if !include_faces {
                        continue;
                    }
                    let read_face = |s: Option<&&str>| {
                        s.and_then(|s| {
                            if s.contains("//") {
                                let mut splits =
                                    s.split('/').filter_map(|n| n.parse::<usize>().ok());
                                splits
                                    .next()
                                    .and_then(|vi| vertexes.get(vi - 1))
                                    .zip(uvs.get(0))
                                    .zip(splits.next().and_then(|ni| normals.get(ni - 1)))
                                    .map(|((v, uv), n)| (*v, *n, *uv))
                            } else {
                                let mut splits =
                                    s.split('/').filter_map(|n| n.parse::<usize>().ok());
                                splits
                                    .next()
                                    .and_then(|vi| vertexes.get(vi - 1))
                                    .zip(splits.next().and_then(|uvi| uvs.get(uvi - 1)))
                                    .zip(splits.next().and_then(|ni| normals.get(ni - 1)))
                                    .map(|((v, uv), n)| (*v, *n, *uv))
                            }
                        })
                    };
                    let a = read_face(parts.get(1));
                    let b = read_face(parts.get(2));
                    let c = read_face(parts.get(3));
                    if let (Some(a), Some(b), Some(c)) = (a, b, c) {
                        let face = builder.build_face(&context, a, b, c)?;
                        faces.push(face);
                    } else {
                        return Err(format!("unable to parse face: {}", line))?;
                    }
                }
                Some("o") | Some("g") => {
                    if let Some(group_name) = parts.get(1) {
                        context.group_name = Some(group_name.to_string());
                        include_faces = builder.include_group(&context);
                    }
                }
                Some("usemtl") => {
                    if let Some(material_name) = parts.get(1) {
                        context.material_name = Some(material_name.to_string());
                    }
                }
                Some("mtllib") => {
                    let material_file = parts[1..].join(" ");
                    context.material_library = Some(path.with_file_name(material_file));
                    builder.load_materials(&context);
                }
                _ => (),
            }
        }

        Ok(faces)
    }
}
