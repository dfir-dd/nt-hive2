use binread::BinRead;
use derive_getters::Getters;

use crate::Offset;

use super::FileType;

/// this data structure follows the documentation found at
/// <https://github.com/msuhanov/regf/blob/master/Windows%20registry%20file%20format%20specification.md#format-of-primary-files>
#[allow(dead_code)]
#[derive(BinRead, Debug, Clone, Default, Getters)]
#[br(magic = b"regf", import(expected_file_type: FileType))]
pub struct HiveBaseBlock {
    /// This number is incremented by 1 in the beginning of a write operation on the primary file
    primary_sequence_number: u32,

    /// This number is incremented by 1 at the end of a write operation on the primary file, a primary sequence number and a secondary sequence number should be equal after a successful write operation
    secondary_sequence_number: u32,

    /// FILETIME (UTC)
    timestamp: u64,

    /// Major version of a hive writer
    #[br(assert(major_version==1))]
    major_version: u32,

    /// Minor version of a hive writer
    #[br(assert(vec![3, 4, 5, 6].contains(&minor_version)))]
    minor_version: u32,

    /// 0 means primary file
    #[br(assert(file_type == expected_file_type))]
    file_type: FileType,

    /// 1 means direct memory load
    #[br(assert(file_format==1))]
    file_format: u32,

    /// Offset of a root cell in bytes, relative from the start of the hive
    /// bins data
    root_cell_offset: Offset,

    /// Size of the hive bins data in bytes
    #[br(assert(data_size%4096 == 0))]
    data_size: u32,

    /// Logical sector size of the underlying disk in bytes divided by 512
    clustering_factor: u32,

    /// UTF-16LE string (contains a partial file path to the primary file, or a
    /// file name of the primary file), used for debugging purposes
    file_name: [u16; 32],

    #[br(count = 99)]
    padding_1: Vec<u32>,

    /// XOR-32 checksum of the previous 508 bytes
    pub checksum: u32,

    /// RESERVED, read only if this is not a transaction log file
    #[br(count = 0x37E, if(file_type == FileType::HiveFile))]
    padding_2: Vec<u32>,

    /// This field has no meaning on a disk, read only if this is not a transaction log file
    #[br(if(file_type == FileType::HiveFile))]
    boot_type: Option<u32>,

    /// This field has no meaning on a disk, read only if this is not a transaction log file
    #[br(if(file_type == FileType::HiveFile))]
    boot_recover: Option<u32>,
}

impl HiveBaseBlock {
    pub fn is_dirty(&self) -> bool {
        self.primary_sequence_number == self.secondary_sequence_number
    }
}
