use crate::error::{ErrorKind, Error, Result};
use std::io::BufRead; // for BufReader.lines
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct XYZParticle<T> {
    pub name : std::string::String,
    pub x    : T,
    pub y    : T,
    pub z    : T,
}

impl<T> XYZParticle<T> {
    pub fn new(name: std::string::String, x: T, y: T, z: T) -> Self {
        XYZParticle::<T>{name: name, x: x, y: y, z: z}
    }
}

// "H 1.00 1.00 1.00" -> XYZParticle
impl<T: FromStr<Err = std::num::ParseFloatError>> FromStr for XYZParticle<T> {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self> {
        let elems: std::vec::Vec<&str> = line.split_whitespace().collect();

        if elems.len() != 4 {
            return Err(Error::new(failure::Context::new(ErrorKind::Format{
                error: format!("invalid XYZ format: {}", line.to_string())
            })));
        }

        let name = elems[0].to_string();
        let x    = elems[1].parse::<T>()?;
        let y    = elems[2].parse::<T>()?;
        let z    = elems[3].parse::<T>()?;
        Ok(XYZParticle::<T>::new(name, x, y, z))
    }
}

pub struct XYZReader<R> {
    bufreader: std::io::BufReader<R>,
}

impl<R: std::io::Read> XYZReader<R> {
    pub fn new(inner: R) -> Self {
        XYZReader::<R>{bufreader: std::io::BufReader::new(inner)}
    }

    pub fn read_snapshot<T: FromStr<Err = std::num::ParseFloatError>>(&mut self)
        -> Result<std::vec::Vec<XYZParticle<T>>> {

        let mut line = std::string::String::new();

        self.bufreader.read_line(&mut line)?;
        let num = line.trim().parse::<usize>()?;
        line.clear();

        // comment line
        self.bufreader.read_line(&mut line)?;
        line.clear();

        let mut snapshot = std::vec::Vec::with_capacity(num);
        for _ in 0 .. num {
            self.bufreader.read_line(&mut line)?;
            snapshot.push(line.parse::<XYZParticle<T>>()?);
            line.clear();
        }
        Ok(snapshot)
    }
}

pub fn open(fname: &str) -> Result<XYZReader<std::fs::File>> {
    let file = std::fs::File::open(fname)?;
    Ok(XYZReader::new(file))
}
