//! Input and output about xyz format file.
//!
//! # example
//! ```no_run
//! use trajan::xyz::XYZReader;
//! let reader = XYZReader::open_pos("example.xyz").unwrap().f64();
//! for snapshot in reader {
//!     println!("{} particles in a snapshot", snapshot.particles.len());
//! }
//! ```
use crate::error::{Error, Result};
use crate::particle::{Attribute, Particle};
use crate::snapshot::Snapshot;
use crate::coordinate::{CoordKind, Coordinate};
use std::io::{BufRead, Write}; // to use read_line

/// Particle contained in a xyz file.
///
/// It may have not only `Position`, but also `Velocity` or `Force` because it
/// contains `Coordinate` defined in this library. By default, when you read a
/// line, it becomes `Position` as described in the following way.
/// ```
/// use trajan::xyz::XYZParticle;
/// let xyz = "H 1.00 1.00 1.00".parse::<XYZParticle<f32>>().unwrap();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct XYZParticle<T> {
    /// name of this particle.
    pub name : std::string::String,
    /// coordinate of this particle.
    pub xyz  : Coordinate<T>,
}

impl<T> XYZParticle<T>
where
    T: std::str::FromStr,
    Error: std::convert::From<<T as std::str::FromStr>::Err>
{
    /// construct XYZParticle.
    pub fn new(name: std::string::String, xyz: Coordinate<T>) -> Self {
        XYZParticle{name: name, xyz: xyz}
    }

    // "H 1.00 1.00 1.00" -> XYZParticle
    fn from_line(line: &str, kind: CoordKind) -> Result<Self> {
        let elems: std::vec::Vec<&str> = line.split_whitespace().collect();

        if elems.len() != 4 {
            return Err(Error::invalid_format(
                format!("invalid XYZ format: {}", line)
            ));
        }

        let name = elems[0].to_string();
        let x    = elems[1].parse()?;
        let y    = elems[2].parse()?;
        let z    = elems[3].parse()?;

        Ok(XYZParticle::new(name, Coordinate::build(kind, x, y, z)))
    }
}

impl<T> std::str::FromStr for XYZParticle<T>
where
    T: std::str::FromStr,
    Error: std::convert::From<<T as std::str::FromStr>::Err>
{
    type Err = Error;
    /// read xyz line such as "H   1.00 1.00 1.00" as a position of particle.
    fn from_str(line: &str) -> Result<Self> {
         Self::from_line(line, CoordKind::Position)
    }
}

impl<T:std::fmt::Display> std::fmt::Display for XYZParticle<T> {
    /// Display xyz line like "H   1.00 1.00 1.00". The width of the fields
    /// are fixed.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:8} {:.16} {:.16} {:.16}",
               self.name, self.xyz[0], self.xyz[1], self.xyz[2])
    }
}

impl<T: nalgebra::Scalar> Particle<T> for XYZParticle<T> {
    type Value = T;
    fn mass(&self) -> Option<T> {
        None
    }
    fn pos(&self) -> Option<nalgebra::Vector3<T>> {
        return if let Coordinate::Position{x, y, z} = self.xyz {
            Some(nalgebra::Vector3::new(x, y, z))
        } else {
            None
        }
    }
    fn vel(&self) -> Option<nalgebra::Vector3<T>> {
        return if let Coordinate::Velocity{x, y, z} = self.xyz {
            Some(nalgebra::Vector3::new(x, y, z))
        } else {
            None
        }
    }
    fn force(&self) -> Option<nalgebra::Vector3<T>> {
        return if let Coordinate::Force{x, y, z} = self.xyz {
            Some(nalgebra::Vector3::new(x, y, z))
        } else {
            None
        }
    }
    fn attribute(&self, name: &str) -> Option<Attribute> {
        return match name {
            "name" => Some(Attribute::String(self.name.clone())),
            _ => None,
        }
    }
}

/// Contains a snapshot in XYZ trajectory file.
#[derive(Debug, Clone, PartialEq)]
pub struct XYZSnapshot<T> {
    /// Comment for the snapshot (the second line in the snapshot).
    /// The line feed at the end of the line is trimmed.
    pub comment:   std::string::String,
    /// Vec of particles contained in the snapshot.
    pub particles: std::vec::Vec<XYZParticle<T>>,
}

impl<T> XYZSnapshot<T> {
    /// Constructs snapshot.
    pub fn new(comment: std::string::String,
               particles: std::vec::Vec<XYZParticle<T>>) -> Self {
        XYZSnapshot{comment: comment, particles: particles}
    }

    /// Gets CoordKind in the XYZSnapshot. Returns None if the snapshot does not
    /// have any particles because the coordinate kind cannot be determined
    /// without particle.
    pub fn which(&self) -> std::option::Option<CoordKind> {
        self.particles.first().map(|p| p.xyz.which())
    }
}

impl<T> std::ops::Index<usize> for XYZSnapshot<T> {
    type Output = XYZParticle<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.particles[index]
    }
}

impl<T: nalgebra::Scalar> Snapshot<T> for XYZSnapshot<T> {
    type Value = T;
    fn len(&self)  -> usize {
        self.particles.len()
    }
    fn masses(&self) -> std::option::Option<std::vec::Vec<T>> {
        None
    }
    fn positions(&self)
        -> std::option::Option<std::vec::Vec<nalgebra::Vector3<T>>>
    {
        self.particles.iter()
            .map(|p| p.pos())
            .collect::<std::option::Option<std::vec::Vec<_>>>()
    }
    fn velocities(&self)
        -> std::option::Option<std::vec::Vec<nalgebra::Vector3<T>>>
    {
        self.particles.iter()
            .map(|p| p.vel())
            .collect::<std::option::Option<std::vec::Vec<_>>>()
    }
    fn forces(&self)
        -> std::option::Option<std::vec::Vec<nalgebra::Vector3<T>>>
    {
        self.particles.iter()
            .map(|p| p.force())
            .collect::<std::option::Option<std::vec::Vec<_>>>()
    }
    fn attributes(&self, name: &str)
        -> std::option::Option<std::vec::Vec<Attribute>>
    {
        self.particles.iter()
            .map(|p| p.attribute(name))
            .collect::<std::option::Option<std::vec::Vec<_>>>()
    }
}

/// Reads XYZSnapshot.
///
/// It can be used as a iterator that reads snapshots until it reaches to the
/// EOF.
///
/// When constructing reader, the CoordKind that represents which kind of
/// coordinate is contained in a file is needed to be provided.
///
/// Also, the precision of the floating point that is used to contain the data
/// is also required. To specify the precision, you can use `.f64()` and
/// `.f32()` functions.
///
/// ```no_run
/// use trajan::xyz::XYZReader;
/// let reader = XYZReader::open_pos("example.xyz").unwrap().f64();
/// for snapshot in reader {
///     println!("{} particles in a snapshot", snapshot.particles.len());
/// }
/// ```
pub struct XYZReader<T, R> {
    pub kind: CoordKind,
    bufreader: std::io::BufReader<R>,
    _marker: std::marker::PhantomData<T>,
}

impl<T, R> XYZReader<T, R>
where
    R: std::io::Read,
    T: std::str::FromStr,
    Error: std::convert::From<<T as std::str::FromStr>::Err>
{
    /// constructing XYZReader.
    pub fn new(kind: CoordKind, inner: R) -> Self {
        XYZReader::<T, R>{
            kind: kind,
            bufreader: std::io::BufReader::new(inner),
            _marker: std::marker::PhantomData
        }
    }

    /// Reads one snapshot from underlying `R: std::io::Read`.
    /// Fails if the file is formatted in an invalid way or reaches to the end.
    pub fn read_snapshot(&mut self) -> Result<XYZSnapshot<T>> {
        let mut line = std::string::String::new();

        self.bufreader.read_line(&mut line)?;
        let num = line.trim().parse::<usize>()?;
        line.clear();

        // comment line
        self.bufreader.read_line(&mut line)?;
        let comment = line.trim().to_string();
        line.clear();

        let mut particles = std::vec::Vec::with_capacity(num);
        for _ in 0 .. num {
            self.bufreader.read_line(&mut line)?;
            particles.push(XYZParticle::from_line(line.as_str(), self.kind)?);
            line.clear();
        }
        Ok(XYZSnapshot::new(comment, particles))
    }
}

impl<T> XYZReader<T, std::fs::File>
where
    T: std::str::FromStr,
    Error: std::convert::From<<T as std::str::FromStr>::Err>
{
    /// Opens file and constructs XYZReader by using the file.
    pub fn open<P>(kind: CoordKind, path: P) -> Result<Self>
    where
        P: std::convert::AsRef<std::path::Path>
    {
        let f = std::fs::File::open(path)?;
        Ok(XYZReader::<T, std::fs::File>{
            kind: kind,
            bufreader: std::io::BufReader::new(f),
            _marker: std::marker::PhantomData
        })
    }

    /// Opens file and constructs XYZReader by using the file.
    /// The coordinate is considered to be Position.
    pub fn open_pos<P>(path: P) -> Result<Self>
    where
        P: std::convert::AsRef<std::path::Path>
    {
        let f = std::fs::File::open(path)?;
        Ok(XYZReader::<T, std::fs::File>{
            kind: CoordKind::Position,
            bufreader: std::io::BufReader::new(f),
            _marker: std::marker::PhantomData
        })
    }
    /// Opens file and constructs XYZReader by using the file.
    /// The coordinate is considered to be Velocity.
    pub fn open_vel<P>(path: P) -> Result<Self>
    where
        P: std::convert::AsRef<std::path::Path>
    {
        let f = std::fs::File::open(path)?;
        Ok(XYZReader::<T, std::fs::File>{
            kind: CoordKind::Velocity,
            bufreader: std::io::BufReader::new(f),
            _marker: std::marker::PhantomData
        })
    }
    /// Opens file and constructs XYZReader by using the file.
    /// The coordinate is considered to be Force.
    pub fn open_force<P>(path: P) -> Result<Self>
    where
        P: std::convert::AsRef<std::path::Path>
    {
        let f = std::fs::File::open(path)?;
        Ok(XYZReader::<T, std::fs::File>{
            kind: CoordKind::Force,
            bufreader: std::io::BufReader::new(f),
            _marker: std::marker::PhantomData
        })
    }
}

/// methods for explicitly specialized type, f32.
impl<R> XYZReader<f32, R> {
    /// An empty function that does nothing.
    /// It is provided in order to set type of `XYZReader` without explicitly write
    /// complicated type parameter, such that
    /// ```no_run
    /// use trajan::xyz::XYZReader;
    /// let r = XYZReader::<f32, _>::open_pos("example.xyz").unwrap();
    /// //                 ^^^^^^^^^^ why we need the second `_`?
    /// ```
    /// By implementing this dummy function, rustc can deduce the type in the
    /// following, simpler and easier way.
    /// ```no_run
    /// use trajan::xyz::XYZReader;
    /// let r = XYZReader::open_pos("hoge").unwrap().f32();
    /// //                                          ^^^^^^ clear!
    /// ```
    pub fn f32(self) -> Self {self}
}
/// methods for explicitly specialized type, f64.
impl<R> XYZReader<f64, R> {
    /// An empty function that does nothing.
    /// It is provided in order to set type of `XYZReader` without explicitly write
    /// complicated type parameter, such that
    /// ```no_run
    /// use trajan::xyz::XYZReader;
    /// let r = XYZReader::<f64, _>::open_pos("hoge").unwrap();
    /// //               ^^^^^^^^^^ why we need the second `_`?
    /// ```
    /// By implementing this dummy function, rustc can deduce the type in the
    /// following, simpler and easier way.
    /// ```no_run
    /// use trajan::xyz::XYZReader;
    /// let r = XYZReader::open_pos("hoge").unwrap().f64();
    /// //                                          ^^^^^^ clear!
    /// ```
    pub fn f64(self) -> Self {self}
}

/// Enables XYZReader to be used as a Iterator of XYZSnapShot.
impl<T, R> std::iter::Iterator for XYZReader<T, R>
where
    R: std::io::Read,
    T: std::str::FromStr,
    Error: std::convert::From<<T as std::str::FromStr>::Err>
{
    type Item = XYZSnapshot<T>;
    fn next(&mut self) -> std::option::Option<Self::Item> {
        self.read_snapshot().ok()
    }
}

/// Writes XYZSnapshot.
///
/// ```no_run
/// use trajan::xyz::{XYZReader, XYZWriter};
/// let reader     = XYZReader::open_pos("example.xyz").unwrap().f64();
/// let mut writer = XYZWriter::new(std::io::stdout());
/// for snapshot in reader {
///     writer.write_snapshot(&snapshot).unwrap();
/// }
/// ```
pub struct XYZWriter<W: std::io::Write> {
    bufwriter: std::io::BufWriter<W>,
}

impl<W: std::io::Write> XYZWriter<W> {
    /// Constructs XYZReader.
    pub fn new(inner: W) -> Self {
        XYZWriter{
            bufwriter: std::io::BufWriter::new(inner),
        }
    }

    /// writes a snapshot.
    pub fn write_snapshot<T>(&mut self, ss: &XYZSnapshot<T>) -> Result<()>
    where
        T: std::fmt::Display
    {
        self.bufwriter.write(ss.particles.len().to_string().as_bytes())?;
        self.bufwriter.write(b"\n")?;
        self.bufwriter.write(ss.comment.as_bytes())?;
        self.bufwriter.write(b"\n")?;
        for particle in &ss.particles {
            self.bufwriter.write(particle.to_string().as_bytes())?;
            self.bufwriter.write(b"\n")?;
        }
        Ok(())
    }
}

impl XYZWriter<std::fs::File> {
    /// opens a file in path and construct XYZWriter using the file.
    pub fn open<P>(path: P) -> Result<Self>
    where
        P: std::convert::AsRef<std::path::Path>
    {
        let f = std::fs::File::open(path)?;
        Ok(XYZWriter{bufwriter: std::io::BufWriter::new(f)})
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_xyz_particle() {
        let p = XYZParticle::new(
            "H".to_string(), Coordinate::Position{x:1.0, y:2.0, z:3.0});
        assert_eq!(p.name, "H");
        assert_eq!(p.xyz,  Coordinate::Position{x:1.0, y:2.0, z:3.0});
    }
    #[test]
    fn access_xyz_particle() {
        let p = XYZParticle::new(
            "H".to_string(), Coordinate::Position{x:1.0, y:2.0, z:3.0});

        assert_eq!(p.mass(),  None);
        assert_eq!(p.pos(),   Some(nalgebra::Vector3::new(1.0, 2.0, 3.0)));
        assert_eq!(p.vel(),   None);
        assert_eq!(p.force(), None);

        if let Attribute::String(name) = p.attribute("name").unwrap() {
            assert_eq!(name, "H");
        } else {
            assert!(false);
        }
    }
    #[test]
    fn read_xyz_line() {
        {
            let p = XYZParticle::from_line("H 1.0 2.0 3.0", CoordKind::Position).unwrap();
            assert_eq!(p.name, "H");
            assert_eq!(p.xyz,  Coordinate::Position{x:1.0, y:2.0, z:3.0});
        }
        {
            let p = "H 1.0 2.0 3.0".parse::<XYZParticle<f64>>().unwrap();
            assert_eq!(p.name, "H");
            assert_eq!(p.xyz,  Coordinate::Position{x:1.0, y:2.0, z:3.0});
        }
    }
    #[test]
    fn construct_xyz_snapshot() {
        let empty = XYZSnapshot::<f64>::new("test".to_string(), vec![]);
        assert_eq!(empty.comment, "test");
        assert_eq!(empty.which(), None);
        assert!(empty.particles.is_empty());

        let s = XYZSnapshot::<f64>::new("test".to_string(), vec![
            "H 1.0 2.0 3.0".parse().unwrap(),
            "C 3.0 2.0 1.0".parse().unwrap(),
        ]);
        assert_eq!(s.comment, "test");
        assert_eq!(s.which(), Some(CoordKind::Position));
        assert_eq!(s.particles.len(), 2);
    }

    #[test]
    fn read_xyz() {
        let contents: &[u8] = b"\
            2
            t = 1
            H 1.0 2.0 3.0
            C 3.0 2.0 1.0
            2
            t = 2
            H 1.1 2.1 3.1
            C 3.1 2.1 1.1
            2
            t = 3
            H 1.2 2.2 3.2
            C 3.2 2.2 1.2"
            ;
        let mut reader = XYZReader::new(CoordKind::Position, contents).f32();
        let s1 = reader.read_snapshot().unwrap();
        let s2 = reader.read_snapshot().unwrap();
        let s3 = reader.read_snapshot().unwrap();

        assert_eq!(s1.comment, "t = 1");
        assert_eq!(s2.comment, "t = 2");
        assert_eq!(s3.comment, "t = 3");

        assert_eq!(s1.particles[0].name, "H");
        assert_eq!(s1.particles[1].name, "C");
        assert_eq!(s2.particles[0].name, "H");
        assert_eq!(s2.particles[1].name, "C");
        assert_eq!(s3.particles[0].name, "H");
        assert_eq!(s3.particles[1].name, "C");

        assert_eq!(s1.particles[0].xyz, Coordinate::Position{x:1.0,y:2.0,z:3.0});
        assert_eq!(s1.particles[1].xyz, Coordinate::Position{x:3.0,y:2.0,z:1.0});
        assert_eq!(s2.particles[0].xyz, Coordinate::Position{x:1.1,y:2.1,z:3.1});
        assert_eq!(s2.particles[1].xyz, Coordinate::Position{x:3.1,y:2.1,z:1.1});
        assert_eq!(s3.particles[0].xyz, Coordinate::Position{x:1.2,y:2.2,z:3.2});
        assert_eq!(s3.particles[1].xyz, Coordinate::Position{x:3.2,y:2.2,z:1.2});
    }
    #[test]
    fn write_xyz() {
        let s1 = XYZSnapshot::<f32>::new("test".to_string(), vec![
            "H 1.0 2.0 3.0".parse().unwrap(),
            "C 3.0 2.0 1.0".parse().unwrap(),
        ]);
        let write_result = {
            let mut buffer = Vec::new();
            {
                let mut writer = XYZWriter::new(&mut buffer);
                writer.write_snapshot(&s1).unwrap();
            }
            buffer
        };
        eprintln!("{:?}", write_result);
        let mut reader = XYZReader::new(CoordKind::Position, write_result.as_slice());
        let s2 = reader.read_snapshot().unwrap();

        assert_eq!(s1, s2);
    }
}

