// Stub — implementation not yet present. Tests will fail (RED).
pub use error::{Result, VhdxError};
pub use reader::VhdxReader;

pub const FILE_MAGIC: &[u8; 8] = b"vhdxfile";

pub mod header {
    pub fn crc32c(_data: &[u8]) -> u32 {
        unimplemented!()
    }
    pub const HEADER1_OFFSET: u64 = 0x0001_0000;
    pub const HEADER2_OFFSET: u64 = 0x0002_0000;
    pub const HEADER_SIGNATURE: &[u8; 4] = b"head";
    pub const HEADER_SIZE: usize = 4096;
    pub const REGION_TABLE1_OFFSET: u64 = 0x0003_0000;
    pub const REGION_TABLE2_OFFSET: u64 = 0x0004_0000;
}

pub mod metadata {
    pub const GUID_FILE_PARAMETERS: [u8; 16] = [0u8; 16];
    pub const GUID_LOGICAL_SECTOR_SIZE: [u8; 16] = [0u8; 16];
    pub const GUID_PARENT_LOCATOR: [u8; 16] = [0u8; 16];
    pub const GUID_PHYSICAL_SECTOR_SIZE: [u8; 16] = [0u8; 16];
    pub const GUID_VIRTUAL_DISK_ID: [u8; 16] = [0u8; 16];
    pub const GUID_VIRTUAL_DISK_SIZE: [u8; 16] = [0u8; 16];
    pub const METADATA_TABLE_SIGNATURE: &[u8; 8] = b"metadata";
}

pub mod region {
    pub const BAT_GUID: [u8; 16] = [0u8; 16];
    pub const MB: u64 = 0x0010_0000;
    pub const METADATA_GUID: [u8; 16] = [0u8; 16];
    pub const REGION_ENTRY_SIZE: usize = 32;
    pub const REGION_TABLE_CRC_COVERAGE: usize = 65536;
    pub const REGION_TABLE_SIGNATURE: &[u8; 4] = b"regi";
}

mod error {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum VhdxError {
        #[error("not a VHDX file (bad magic)")]
        BadMagic,
        #[error("no valid VHDX header found")]
        NoValidHeader,
        #[error("region table not found or invalid")]
        InvalidRegionTable,
        #[error("BAT region not found in region table")]
        BatRegionMissing,
        #[error("metadata region not found in region table")]
        MetadataRegionMissing,
        #[error("required metadata item missing: {0}")]
        MetadataMissing(&'static str),
        #[error("metadata value is outside valid range: {0}")]
        InvalidMetadata(&'static str),
        #[error("container is too small to be a valid VHDX (minimum {0} bytes required)")]
        ContainerTooSmall(u64),
        #[error("region or BAT file offset is outside the container bounds")]
        OffsetOutOfBounds,
        #[error("BAT entry file offset calculation overflows u64")]
        AddressOverflow,
        #[error("sector out of range (sector {sector}, virtual disk size {size})")]
        SectorOutOfRange { sector: u64, size: u64 },
        #[error("BAT entry not present for sector {0}")]
        BlockNotPresent(u64),
        #[error("I/O error: {0}")]
        Io(#[from] std::io::Error),
        #[error("VHDX has a parent locator (differencing disk not supported)")]
        DifferencingNotSupported,
    }

    pub type Result<T> = std::result::Result<T, VhdxError>;
}

mod reader {
    use crate::error::{Result, VhdxError};
    use std::io::{self, Read, Seek, SeekFrom};

    #[derive(Debug)]
    pub struct VhdxReader {
        _data: Vec<u8>,
    }

    impl VhdxReader {
        pub fn open(path: &std::path::Path) -> Result<Self> {
            let data = std::fs::read(path)?;
            Self::from_bytes(data)
        }

        pub fn from_bytes(_data: Vec<u8>) -> Result<Self> {
            Err(VhdxError::BadMagic)
        }

        pub fn virtual_disk_size(&self) -> u64 {
            0
        }

        pub fn logical_sector_size(&self) -> u32 {
            0
        }
    }

    impl Read for VhdxReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::Other, "not implemented"))
        }
    }

    impl Seek for VhdxReader {
        fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
            Err(io::Error::new(io::ErrorKind::Other, "not implemented"))
        }
    }
}
