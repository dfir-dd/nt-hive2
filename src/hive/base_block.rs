use std::io::Write;

use binread::{count, BinRead};
use binwrite::{write_track::WriteTrack, BinWrite, WriterOption};
use byteorder::{LittleEndian, WriteBytesExt};
use derive_getters::Getters;

use crate::Offset;

use super::FileType;

pub const BASEBLOCK_SIZE: usize = 4096;

#[derive(Default, Debug, Clone)]
struct CalculatedChecksum(u32);

impl AsRef<u32> for CalculatedChecksum {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl BinRead for CalculatedChecksum {
    type Args = ();

    fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        options: &binread::ReadOptions,
        _: Self::Args,
    ) -> binread::prelude::BinResult<Self> {

        reader.seek(std::io::SeekFrom::End(0))?;
        reader.seek(std::io::SeekFrom::Start(0))?;

        let data: Vec<u32> = count(127)(reader, options, ())?;
        
        let checksum = match data.into_iter().fold(0, |acc, x| acc ^ x) {
            0xffff_ffff => 0xffff_fffe,
            0 => 1,
            sum => sum,
        };
        Ok(Self(checksum))
    }
}

/// this data structure follows the documentation found at
/// <https://github.com/msuhanov/regf/blob/master/Windows%20registry%20file%20format%20specification.md#format-of-primary-files>
#[allow(dead_code)]
#[derive(BinRead, Debug, Clone, Default, Getters, BinWrite)]
#[br(import(expected_file_type: FileType))]
pub struct HiveBaseBlock {
    #[br(restore_position)]
    #[binwrite(ignore)]
    calculated_checksum: CalculatedChecksum,

    /// magic number. This is not specified as `magic` attribute for BinRead, because this needs to be
    /// accessible by BinWrite
    ///
    /// Offset: 0x0000
    #[br(assert(magic == "regf".as_bytes()))]
    magic: [u8; 4],

    /// This number is incremented by 1 in the beginning of a write operation on the primary file
    ///
    /// Offset: 0x0004
    primary_sequence_number: u32,

    /// This number is incremented by 1 at the end of a write operation on the primary file, a primary sequence number and a secondary sequence number should be equal after a successful write operation
    ///
    /// Offset: 0x0008
    secondary_sequence_number: u32,

    /// FILETIME (UTC)
    ///
    /// Offset: 0x000c
    timestamp: u64,

    /// Major version of a hive writer
    ///
    /// Offset: 0x0014
    #[br(assert(major_version==1))]
    major_version: u32,

    /// Minor version of a hive writer
    ///
    /// Offset: 0x0018
    #[br(assert([3, 4, 5, 6].contains(&minor_version)))]
    minor_version: u32,

    /// 0 means primary file
    ///
    /// Offset: 0x001c
    #[br(assert(file_type == expected_file_type))]
    file_type: FileType,

    /// 1 means direct memory load
    ///
    /// Offset: 0x0020
    #[br(assert(file_format==1))]
    file_format: u32,

    /// Offset of a root cell in bytes, relative from the start of the hive
    /// bins data
    ///
    /// Offset: 0x0024
    root_cell_offset: Offset,

    /// Size of the hive bins data in bytes
    ///
    /// Offset: 0x0028
    #[br(assert(data_size%4096 == 0, "actual value is {data_size}"))]
    data_size: u32,

    /// Logical sector size of the underlying disk in bytes divided by 512
    ///
    /// Offset: 0x002c
    clustering_factor: u32,

    /// UTF-16LE string (contains a partial file path to the primary file, or a
    /// file name of the primary file), used for debugging purposes
    #[br(count = 32)]
    file_name: Vec<u16>,

    #[br(count = 99)]
    padding_1: Vec<u32>,

    /// XOR-32 checksum of the previous 508 bytes
    #[br(assert(calculated_checksum.as_ref() == &checksum, "expected checksum of 0x{:08x}, but found 0x{checksum:08x} instead", calculated_checksum.as_ref()))]
    pub checksum: u32,

    /// RESERVED, read only if this is not a transaction log file
    #[br(count = 0x37E, if(file_type == FileType::HiveFile))]
    padding_2: Vec<u32>,

    /// This field has no meaning on a disk, read only if this is not a transaction log file
    #[br(if(file_type == FileType::HiveFile))]
    #[binwrite(with(write_opt_u32))]
    boot_type: Option<u32>,

    /// This field has no meaning on a disk, read only if this is not a transaction log file
    #[br(if(file_type == FileType::HiveFile))]
    #[binwrite(with(write_opt_u32))]
    boot_recover: Option<u32>,
}

fn write_opt_u32<W: Write>(
    value: &std::option::Option<u32>,
    write_track: &mut WriteTrack<&mut W>,
    _option: &&WriterOption,
) -> std::result::Result<(), std::io::Error> {
    match value {
        Some(v) => write_track.write_u32::<LittleEndian>(*v),
        None => write_track.write_u32::<LittleEndian>(0),
    }
}

impl HiveBaseBlock {
    pub fn is_dirty(&self) -> bool {
        self.primary_sequence_number != self.secondary_sequence_number
    }

    pub fn set_sequence_number(&mut self, sequence_number: u32) {
        assert!(sequence_number >= self.primary_sequence_number);
        assert!(sequence_number >= self.secondary_sequence_number);

        // patch out the old sequence numbers
        self.checksum ^= (!self.primary_sequence_number) ^ (!self.secondary_sequence_number);

        self.primary_sequence_number = sequence_number;
        self.secondary_sequence_number = sequence_number;

        // add the new sequence numbers to the checksum
        self.checksum ^= (self.primary_sequence_number) ^ (self.secondary_sequence_number);
    }
}
