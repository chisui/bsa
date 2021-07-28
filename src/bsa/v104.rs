use std::io::{Read, Seek, Write, Result};
use std::str;
use std::fmt;
use bytemuck::{Pod, Zeroable};
use enumflags2::{bitflags, BitFlags};

use super::v103::{V10XHeader, ToArchiveBitFlags};
use super::bin::{self, Readable};
use super::version::Version;
use super::hash::Hash;
use super::archive::{Bsa, BsaDir, BsaFile};
pub use super::v103::{FileFlag, FolderRecord, RawHeader, Has, BZString, extract};


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ArchiveFlag {
    #[doc = "The game may not load a BSA without this bit set."]
    pub IncludeDirectoryNames = 0x1,
    #[doc = "The game may not load a BSA without this bit set."]
    pub IncludeFileNames = 0x2,
    #[doc = "This does not mean all files are compressed. It means they are"]
    #[doc = "compressed by default."]
    pub CompressedArchive = 0x4,
    pub RetainDirectoryNames = 0x8,
    pub RetainFileNames = 0x10,
    pub RetainFileNameOffsets = 0x20,
    #[doc = "Hash values and numbers after the header are encoded big-endian."]
    pub Xbox360Archive = 0x40,
    pub RetainStringsDuringStartup = 0x80,
    #[doc = "Embed File Names. Indicates the file data blocks begin with a"]
    #[doc = "bstring containing the full path of the file. For example, in"]
    #[doc = "\"Skyrim - Textures.bsa\" the first data block is"]
    #[doc = "$2B textures/effects/fxfluidstreamdripatlus.dds"]
    #[doc = "($2B indicating the name is 43 bytes). The data block begins"]
    #[doc = "immediately after the bstring."]
    pub EmbedFileNames = 0x100,
    #[doc = "This can only be used with COMPRESSED_ARCHIVE."]
    #[doc = "This is an Xbox 360 only compression algorithm."]
    pub XMemCodec = 0x200,
}


impl ToArchiveBitFlags for ArchiveFlag {
    fn to_archive_bit_flags(bits: u32) -> BitFlags<Self> {
        BitFlags::from_bits_truncate(bits)
    }
}

pub type Header = V10XHeader<ArchiveFlag>;

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct FileRecord {
    pub name_hash: Hash,
    pub size: u32,
    pub offset: u32,
}
impl FileRecord {
    pub fn is_compression_bit_set(&self) -> bool {
        (self.size & 0x40000000) == 0x40000000
    }
}
impl bin::Readable for FileRecord {
    fn read_here<R: Read + Seek>(mut reader: R, _: &()) -> Result<FileRecord> {
        bin::read_struct(&mut reader)
    }
}


pub struct V104(pub Header);
impl Bsa for V104 {
    fn open<R: Read + Seek>(reader: R) -> Result<V104> {
        let header = Header::read(reader, &())?;
        Ok(V104(header))
    }

    fn version(&self) -> Version { Version::V104 }

    fn read_dirs<R: Read + Seek>(&self, _: R) -> Result<Vec<BsaDir>> {
        Ok(vec![])
    }

    fn extract<R: Read + Seek, W: Write>(&self, _: BsaFile, _: W, _: R) -> Result<()> {
        Ok(())
    }
} 
impl fmt::Display for V104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "BSA v104 file, format used by: TES V: Skyrim, Fallout 3 and Fallout: New Vegas")?;
        writeln!(f, "{}", self.0)
    }
}
