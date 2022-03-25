
use crate::Cell;
use crate::Hive;
use crate::NtHiveError;
use crate::Result;
use crate::Offset;
use crate::traits::FromOffset;

use std::borrow::Cow;
use std::io::Read;
use std::io::Seek;
use std::ops::Deref;
use bitflags::bitflags;
use binread::BinResult;
use binread::ReadOptions;
use binread::{BinRead, BinReaderExt};
use encoding_rs::ISO_8859_15;
use encoding_rs::UTF_16LE;

#[derive(BinRead)]
#[br(import(count: u32))]
pub struct KeyValueList {
    #[br(count=count)]
    pub key_value_offsets: Vec<Offset>
}


impl From<Cell<KeyValueList>> for KeyValueList {
    fn from(cell: Cell<KeyValueList>) -> Self {
        cell.into_data()
    }
}

/// Possible data types of the data belonging to a [`KeyValue`].
#[derive(BinRead)]
#[br(repr=u32)]
pub enum KeyValueDataType {
    RegNone = 0x0000_0000,
    RegSZ = 0x0000_0001,
    RegExpandSZ = 0x0000_0002,
    RegBinary = 0x0000_0003,
    RegDWord = 0x0000_0004,
    RegDWordBigEndian = 0x0000_0005,
    RegLink = 0x0000_0006,
    RegMultiSZ = 0x0000_0007,
    RegResourceList = 0x0000_0008,
    RegFullResourceDescriptor = 0x0000_0009,
    RegResourceRequirementsList = 0x0000_000a,
    RegQWord = 0x0000_000b,
}

bitflags! {
    #[allow(non_upper_case_globals)]
    pub struct KeyValueFlags: u16 {
        /// The name is in (extended) ASCII instead of UTF-16LE.
        const VALUE_COMP_NAME = 0x0001;

        /// Is a tombstone value (the flag is used starting from Insider Preview
        /// builds of Windows 10 "Redstone 1"), a tombstone value also has the
        /// Data type field set to REG_NONE, the Data size field set to 0, and
        /// the Data offset field set to 0xFFFFFFFF
        const IsTombstone = 0x0002;
    }
}

#[derive(BinRead)]
#[br(magic = b"vk")]
#[allow(dead_code)]
pub struct KeyValue {
    name_length: u16,
    data_size: u32,
    data_offset: Offset,
    data_type: KeyValueDataType,

    #[br(parse_with=parse_value_flags)]
    flags: KeyValueFlags,
    spare: u16,

    #[br(count=name_length)]
    key_name_string: Vec<u8>,
}

fn parse_value_flags<R: Read + Seek>(reader: &mut R, _ro: &ReadOptions, _: ())
-> BinResult<KeyValueFlags>
{
    let raw_value: u16 = reader.read_le()?;
    Ok(KeyValueFlags::from_bits(raw_value).unwrap())
}

impl KeyValue
{
    pub fn name(&self) -> Result<Cow<str>> {
        let (cow, _, had_errors) = 
        if self.flags.contains(KeyValueFlags::VALUE_COMP_NAME) {
            ISO_8859_15.decode(&self.key_name_string[..])
        } else {
            UTF_16LE.decode(&self.key_name_string[..])
        };

        if had_errors {
            Err(NtHiveError::StringEncodingError)
        } else {
            Ok(cow)
        }
    }
}


impl From<Cell<KeyValue>> for KeyValue {
    fn from(cell: Cell<KeyValue>) -> Self {
        cell.into_data()
    }
}