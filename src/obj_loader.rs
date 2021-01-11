use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub trait ObjGroupFilter {
    fn include_group(&self, group_name: Option<&str>) -> bool;
}

impl ObjGroupFilter for () {
    fn include_group(&self, _group_name: Option<&str>) -> bool {
        true
    }
}

pub struct ObjLoader;

impl ObjLoader {
    pub fn load<
        GF: ObjGroupFilter,
        P: AsRef<Path>,
        FV: FnMut(f32, f32, f32) -> V,
        FN: FnMut(f32, f32, f32) -> N,
        FUV: FnMut(f32, f32) -> UV,
        FF: FnMut((V, N, UV), (V, N, UV), (V, N, UV)) -> F,
        V: Copy,
        N: Copy,
        UV: Copy,
        F,
    >(
        path: P,
        group_filter: &GF,
        mut vertex_fn: FV,
        mut normal_fn: FN,
        mut uv_fn: FUV,
        mut face_fn: FF,
    ) -> Result<Vec<F>, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let mut file = BufReader::new(File::open(path)?);

        let mut vertexes = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut faces = Vec::new();

        let mut line = String::new();
        let mut include_faces = group_filter.include_group(None);
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
                        let vert = vertex_fn(x, y, z);
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
                        let normal = normal_fn(x, y, z);
                        normals.push(normal);
                    } else {
                        return Err(format!("unable to parse normal: {}", line))?;
                    }
                }
                Some("vt") => {
                    let u = parts.get(1).and_then(|n| n.parse().ok());
                    let v = parts.get(2).and_then(|n| n.parse().ok());

                    if let (Some(u), Some(v)) = (u, v) {
                        let uv = uv_fn(u, v);
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
                            let mut splits = s.split('/').filter_map(|n| n.parse::<usize>().ok());
                            splits
                                .next()
                                .and_then(|vi| vertexes.get(vi - 1))
                                .zip(splits.next().and_then(|uvi| uvs.get(uvi - 1)))
                                .zip(splits.next().and_then(|ni| normals.get(ni - 1)))
                                .map(|((v, uv), n)| (*v, *n, *uv))
                        })
                    };
                    let a = read_face(parts.get(1));
                    let b = read_face(parts.get(2));
                    let c = read_face(parts.get(3));
                    if let (Some(a), Some(b), Some(c)) = (a, b, c) {
                        let face = face_fn(a, b, c);
                        faces.push(face);
                    } else {
                        return Err(format!("unable to parse face: {}", line))?;
                    }
                }
                Some("o") => {
                    if let Some(group_name) = parts.get(1) {
                        include_faces = group_filter.include_group(Some(group_name));
                    }
                }
                _ => (),
            }
        }

        Ok(faces)
    }
}
