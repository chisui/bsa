use std::{
    mem::size_of,
    io::{self, BufReader, Read, Write, Seek, Result},
    path::Path,
    fs::File,
    fmt,
};

use thiserror::Error;

use crate::{
    bin,
    magicnumber::MagicNumber,
};


#[derive(Debug, Error)]
#[error("Unsupported Version {0}")]
struct UnsupportedVersion(pub Version);

#[derive(Debug, Error)]
pub enum Unknown {
    #[error("Unknown magic number {0}")]
    MagicNumber(u32),
    #[error("Unknown version {0}")]
    Version(u32),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Version {
    V001, // TES3
    V10X(Version10X),
    V200(u32), // F4 F76
}
impl Version {
    pub fn open<P>(&self, path: P) -> Result<crate::SomeBsaReader<BufReader<File>>>
    where P: AsRef<Path> {
        let file = File::open(path)?;
        let buf = BufReader::new(file);
        self.read(buf)
    }
    pub fn read<R: Read + Seek>(&self, reader: R) -> Result<crate::SomeBsaReader<R>> {
        match self {
            Version::V001 => crate::v001::read(reader).map(crate::SomeBsaReader::V001),
            Version::V10X(v) => v.read(reader),
            Version::V200(_) => Err(io::Error::new(io::ErrorKind::InvalidInput, UnsupportedVersion(*self))),
        }
    }
}
impl From<&Version> for MagicNumber {
    fn from(version: &Version) -> MagicNumber {
        match version {
            Version::V001    => MagicNumber::V001,
            Version::V10X(_) => MagicNumber::V10X,
            Version::V200(_) => MagicNumber::BTDX,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Version10X {
    V103 = 103, // TES4
    V104 = 104, // F3, FNV, TES5
    V105 = 105, // TES5se
}
impl Version10X {
    pub fn read<R: Read + Seek>(&self, reader: R) -> Result<crate::SomeBsaReader<R>> {
        match self {
            Version10X::V103 => crate::v103::read(reader).map(crate::SomeBsaReader::V103),
            Version10X::V104 => crate::v104::read(reader).map(crate::SomeBsaReader::V104),
            Version10X::V105 => crate::v105::read(reader).map(crate::SomeBsaReader::V105),
        }
    }
}
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Version::V001 => write!(f, "v100"),
            Version::V10X(Version10X::V103) => write!(f, "v103"),
            Version::V10X(Version10X::V104) => write!(f, "v104"),
            Version::V10X(Version10X::V105) => write!(f, "v105"),
            Version::V200(v) => write!(f, "BA2 v{:03}", v),
        }
    }
}

impl bin::Writable for Version {
    fn size(&self) -> usize { 
        size_of::<MagicNumber>() + match self {
            Version::V001 => 0,
            Version::V10X(_) => size_of::<Version10X>(),
            Version::V200(_) => size_of::<u32>(),
        }
     }
    fn write_here<W: Write>(&self, mut writer: W) -> Result<()> {
        MagicNumber::from(self).write_here(&mut writer)?;
        match self {
            Version::V001 => Ok(()),
            Version::V200(v) => v.write_here(writer),
            Version::V10X(v) => (*v as u32).write_here(writer),
        }
    }
}
impl bin::Readable for Version {
    fn offset(_: &()) -> Option<usize> {
        Some(0)
    }
    fn read_here<R: Read + Seek>(mut buffer: R, _: &()) -> Result<Self> {
        match MagicNumber::read_here0(&mut buffer)? {
            MagicNumber::V001 => Ok(Version::V001),
            MagicNumber::V10X => {
                let version= u32::read_here0(&mut buffer)?;
                match version {
                    103 => Ok(Version10X::V103),
                    104 => Ok(Version10X::V104),
                    105 => Ok(Version10X::V105),
                    _ => Err(io::Error::new(io::ErrorKind::InvalidData, Unknown::Version(version))),
                }.map(Version::V10X)
            },
            MagicNumber::BTDX => {
                let v = u32::read_here0(&mut buffer)?;
                Ok(Version::V200(v))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bin::test::*;
    use super::*;

    #[test]
    fn write_read_identity_version() {
        for v in [
            Version::V001, 
            Version::V10X(Version10X::V103), 
            Version::V10X(Version10X::V104), 
            Version::V10X(Version10X::V105), 
            Version::V200(12),
        ] {
            write_read_identity(v)
        }
    }
}
