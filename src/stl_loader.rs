use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};

pub struct StlLoader;

impl StlLoader {
    pub fn load_binary<
        P: AsRef<Path>,
        FV: FnMut(f32, f32, f32) -> V,
        FF: FnMut(V, V, V) -> F,
        V: Copy,
        F,
    >(
        path: P,
        mut vertex_fn: FV,
        mut face_fn: FF,
    ) -> Result<Vec<F>, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let mut file = BufReader::new(File::open(path)?);

        let mut header = [0; 80];
        file.read_exact(&mut header)?;

        let tri_count = file.read_u32::<LittleEndian>()?;

        eprintln!("loading stl with {} triangles", tri_count);

        let mut faces = Vec::new();

        for _ in 0..tri_count {
            let _norm_x = file.read_f32::<LittleEndian>()?;
            let _norm_y = file.read_f32::<LittleEndian>()?;
            let _norm_z = file.read_f32::<LittleEndian>()?;

            let a_x = file.read_f32::<LittleEndian>()?;
            let a_y = file.read_f32::<LittleEndian>()?;
            let a_z = file.read_f32::<LittleEndian>()?;

            let b_x = file.read_f32::<LittleEndian>()?;
            let b_y = file.read_f32::<LittleEndian>()?;
            let b_z = file.read_f32::<LittleEndian>()?;

            let c_x = file.read_f32::<LittleEndian>()?;
            let c_y = file.read_f32::<LittleEndian>()?;
            let c_z = file.read_f32::<LittleEndian>()?;

            let a = vertex_fn(a_x, a_y, a_z);
            let b = vertex_fn(b_x, b_y, b_z);
            let c = vertex_fn(c_x, c_y, c_z);

            let face = face_fn(a, b, c);

            faces.push(face);

            let attr_count = file.read_u16::<LittleEndian>()?;

            let mut attrs = vec![0; attr_count as usize];
            file.read_exact(&mut attrs)?;
        }

        Ok(faces)
    }
}
