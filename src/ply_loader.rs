use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

#[derive(Debug, Copy, Clone)]
enum Format {
    Ascii,
    BinaryLE,
    BinaryBE,
}

impl Format {
    fn read_usize<R: Read>(
        &self,
        reader: &mut R,
        kind: DataType,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let value = match self {
            Format::Ascii => {
                let mut buf = [0u8; 1];
                let mut word = String::new();
                loop {
                    reader.read_exact(&mut buf)?;
                    let c = buf[0] as char;
                    if c.is_whitespace() && word.len() > 0 {
                        break;
                    } else if !c.is_whitespace() {
                        word.push(c);
                    }
                }
                match kind {
                    DataType::Char
                    | DataType::Short
                    | DataType::Int
                    | DataType::UChar
                    | DataType::UShort
                    | DataType::UInt => word.parse()?,
                    DataType::Float | DataType::Double => word.parse::<f64>()? as usize,
                }
            }
            Format::BinaryLE => match kind {
                DataType::Char => reader.read_i8()? as usize,
                DataType::UChar => reader.read_u8()? as usize,
                DataType::Short => reader.read_i16::<LittleEndian>()? as usize,
                DataType::UShort => reader.read_u16::<LittleEndian>()? as usize,
                DataType::Int => reader.read_i32::<LittleEndian>()? as usize,
                DataType::UInt => reader.read_u32::<LittleEndian>()? as usize,
                DataType::Float => reader.read_f32::<LittleEndian>()? as usize,
                DataType::Double => reader.read_f64::<LittleEndian>()? as usize,
            },
            Format::BinaryBE => match kind {
                DataType::Char => reader.read_i8()? as usize,
                DataType::UChar => reader.read_u8()? as usize,
                DataType::Short => reader.read_i16::<BigEndian>()? as usize,
                DataType::UShort => reader.read_u16::<BigEndian>()? as usize,
                DataType::Int => reader.read_i32::<BigEndian>()? as usize,
                DataType::UInt => reader.read_u32::<BigEndian>()? as usize,
                DataType::Float => reader.read_f32::<BigEndian>()? as usize,
                DataType::Double => reader.read_f64::<BigEndian>()? as usize,
            },
        };

        Ok(value)
    }

    fn read_f64<R: Read>(
        &self,
        reader: &mut R,
        kind: DataType,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let value = match self {
            Format::Ascii => {
                let mut buf = [0u8; 1];
                let mut word = String::new();
                loop {
                    reader.read_exact(&mut buf)?;
                    let c = buf[0] as char;
                    if c.is_whitespace() && word.len() > 0 {
                        break;
                    } else if !c.is_whitespace() {
                        word.push(c);
                    }
                }
                word.parse()?
            }
            Format::BinaryLE => match kind {
                DataType::Char => reader.read_i8()? as f64,
                DataType::UChar => reader.read_u8()? as f64,
                DataType::Short => reader.read_i16::<LittleEndian>()? as f64,
                DataType::UShort => reader.read_u16::<LittleEndian>()? as f64,
                DataType::Int => reader.read_i32::<LittleEndian>()? as f64,
                DataType::UInt => reader.read_u32::<LittleEndian>()? as f64,
                DataType::Float => reader.read_f32::<LittleEndian>()? as f64,
                DataType::Double => reader.read_f64::<LittleEndian>()? as f64,
            },
            Format::BinaryBE => match kind {
                DataType::Char => reader.read_i8()? as f64,
                DataType::UChar => reader.read_u8()? as f64,
                DataType::Short => reader.read_i16::<BigEndian>()? as f64,
                DataType::UShort => reader.read_u16::<BigEndian>()? as f64,
                DataType::Int => reader.read_i32::<BigEndian>()? as f64,
                DataType::UInt => reader.read_u32::<BigEndian>()? as f64,
                DataType::Float => reader.read_f32::<BigEndian>()? as f64,
                DataType::Double => reader.read_f64::<BigEndian>()? as f64,
            },
        };

        Ok(value)
    }

    fn skip<R: Read>(
        &self,
        reader: &mut R,
        kind: DataType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Format::Ascii => {
                let mut buf = [0u8; 1];
                let mut word = String::new();
                loop {
                    reader.read_exact(&mut buf)?;
                    let c = buf[0] as char;
                    if c.is_whitespace() && word.len() > 0 {
                        break;
                    } else if !c.is_whitespace() {
                        word.push(c);
                    }
                }
            }
            Format::BinaryLE | Format::BinaryBE => match kind {
                DataType::Char | DataType::UChar => {
                    let mut buf = [0u8; 1];
                    reader.read_exact(&mut buf)?;
                }
                DataType::Short | DataType::UShort => {
                    let mut buf = [0u8; 2];
                    reader.read_exact(&mut buf)?;
                }
                DataType::Int | DataType::UInt | DataType::Float => {
                    let mut buf = [0u8; 4];
                    reader.read_exact(&mut buf)?;
                }
                DataType::Double => {
                    let mut buf = [0u8; 8];
                    reader.read_exact(&mut buf)?;
                }
            },
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
enum DataType {
    Char,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Float,
    Double,
}

impl std::str::FromStr for DataType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = match s {
            "char" | "int8" => DataType::Char,
            "uchar" | "uint8" => DataType::UChar,
            "short" | "int16" => DataType::Short,
            "ushort" | "uint16" => DataType::UShort,
            "int" | "int32" => DataType::Int,
            "uint" | "uint32" => DataType::UInt,
            "float" | "float32" => DataType::Float,
            "double" | "float64" => DataType::Double,
            _ => return Err(Error::InvalidFile),
        };

        Ok(t)
    }
}

#[derive(Debug, Clone)]
enum Error {
    InvalidFile,
    InvalidFormat(String, String),
    InvalidProperty(String),
    InvalidElement(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidFile => write!(f, "ply magic number not found"),
            Error::InvalidFormat(format, version) => {
                write!(f, "ply unsupported format found: {} {}", format, version)
            }
            Error::InvalidProperty(line) => {
                write!(f, "ply invalid property: '{}'", line)
            }
            Error::InvalidElement(line) => {
                write!(f, "ply invalid element: '{}'", line)
            }
        }
    }
}
impl std::error::Error for Error {}

#[derive(Debug, Clone)]
struct PlyDescription {
    format: Format,
    elements: Vec<Element>,
}

impl PlyDescription {
    fn new() -> Self {
        PlyDescription {
            format: Format::Ascii,
            elements: Vec::new(),
        }
    }
    fn add_element<S: Into<String>>(&mut self, name: S, count: usize) {
        let elem = Element {
            name: name.into(),
            count,
            properties: Vec::new(),
        };

        self.elements.push(elem);
    }

    fn add_property<S: Into<String>>(&mut self, name: S, kind: DataType) {
        if let Some(elem) = self.elements.last_mut() {
            let prop = Property::Field(name.into(), kind);
            elem.properties.push(prop);
        }
    }

    fn add_property_list<S: Into<String>>(
        &mut self,
        name: S,
        count_kind: DataType,
        property_kind: DataType,
    ) {
        if let Some(elem) = self.elements.last_mut() {
            let prop = Property::List(name.into(), count_kind, property_kind);
            elem.properties.push(prop);
        }
    }
}

#[derive(Debug, Clone)]
struct Element {
    name: String,
    count: usize,
    properties: Vec<Property>,
}

#[derive(Debug, Clone)]
enum Property {
    Field(String, DataType),
    List(String, DataType, DataType),
}

pub struct PlyLoader {}

impl PlyLoader {
    pub fn load<
        P: AsRef<Path>,
        FV: FnMut(f64, f64, f64) -> V,
        FF: FnMut(V, V, V) -> F,
        V: Copy,
        F,
    >(
        path: P,
        mut vertex_fn: FV,
        mut face_fn: FF,
    ) -> Result<Vec<F>, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut line = String::new();
        reader.read_line(&mut line)?;

        if line.trim() != "ply" {
            return Err(Error::InvalidFile)?;
        }

        let mut reading_header = true;
        let mut ply_description = PlyDescription::new();
        while reading_header {
            line.clear();
            reader.read_line(&mut line)?;

            let mut split = line.trim().split(' ');
            let command = split.next();

            match command {
                Some("end_header") => reading_header = false,
                Some("format") => {
                    let format = split.next();
                    let version = split.next();
                    let format = match format.zip(version) {
                        Some(("ascii", "1.0")) => Format::Ascii,
                        Some(("binary_little_endian", "1.0")) => Format::BinaryLE,
                        Some(("binary_big_endian", "1.0")) => Format::BinaryBE,
                        _ => {
                            let format = String::from(format.unwrap_or(""));
                            let version = String::from(version.unwrap_or(""));
                            return Err(Error::InvalidFormat(format, version))?;
                        }
                    };
                    ply_description.format = format;
                }
                Some("comment") => (),
                Some("element") => {
                    let name = split.next();
                    let count: Option<usize> = split.next().and_then(|n| n.parse().ok());
                    let (name, count) = name
                        .zip(count)
                        .ok_or_else(|| Error::InvalidElement(line.to_string()))?;
                    ply_description.add_element(name, count);
                }
                Some("property") => {
                    let kind = split.next();
                    match kind {
                        Some("list") => {
                            let count_kind: Option<DataType> =
                                split.next().and_then(|k| k.parse().ok());
                            let property_kind: Option<DataType> =
                                split.next().and_then(|k| k.parse().ok());
                            let name = split.next();
                            let ((name, count_kind), property_kind) = name
                                .zip(count_kind)
                                .zip(property_kind)
                                .ok_or_else(|| Error::InvalidProperty(line.to_string()))?;
                            ply_description.add_property_list(name, count_kind, property_kind);
                        }
                        Some(kind) => {
                            let kind: Option<DataType> = kind.parse().ok();
                            let name = split.next();
                            let (name, kind) = name
                                .zip(kind)
                                .ok_or_else(|| Error::InvalidProperty(line.to_string()))?;
                            ply_description.add_property(name, kind);
                        }
                        None => (),
                    }
                }
                Some(unknown) => eprintln!("unknown ply header found: '{}'", unknown),
                None => (),
            }
        }

        let mut vertexes = Vec::new();
        let mut faces = Vec::new();

        for element in ply_description.elements {
            let is_vertex = element.name == "vertex";
            let is_face = element.name == "face";

            if is_vertex || is_face {
                vertexes.reserve(element.count);
            }

            for _ in 0..element.count {
                let mut x = None;
                let mut y = None;
                let mut z = None;
                for prop in &element.properties {
                    match prop {
                        Property::Field(name, kind) => match (is_vertex, name.as_str()) {
                            (true, "x") => {
                                x = Some(ply_description.format.read_f64(&mut reader, *kind)?);
                            }
                            (true, "y") => {
                                y = Some(ply_description.format.read_f64(&mut reader, *kind)?);
                            }
                            (true, "z") => {
                                z = Some(ply_description.format.read_f64(&mut reader, *kind)?);
                            }
                            _ => {
                                ply_description.format.skip(&mut reader, *kind)?;
                            }
                        },
                        Property::List(_name, count_kind, value_kind) => {
                            let count = ply_description
                                .format
                                .read_usize(&mut reader, *count_kind)?;
                            if is_face && count == 3 {
                                let a_idx = ply_description
                                    .format
                                    .read_usize(&mut reader, *value_kind)?;
                                let b_idx = ply_description
                                    .format
                                    .read_usize(&mut reader, *value_kind)?;
                                let c_idx = ply_description
                                    .format
                                    .read_usize(&mut reader, *value_kind)?;

                                let face =
                                    face_fn(vertexes[a_idx], vertexes[b_idx], vertexes[c_idx]);

                                faces.push(face);
                            } else {
                                for _ in 0..count {
                                    ply_description.format.skip(&mut reader, *value_kind)?;
                                }
                            }
                        }
                    }
                }

                if is_vertex {
                    if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                        let vert = vertex_fn(x, y, z);
                        vertexes.push(vert);
                    }
                }
            }
        }

        Ok(faces)
    }
}
