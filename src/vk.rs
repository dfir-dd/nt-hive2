use crate::util::parse_string;
use crate::util::SizedVec;
use crate::Cell;
use crate::Hive;
use crate::Offset;
use crate::Result;

use binread::BinResult;
use binread::PosValue;
use binread::ReadOptions;
use binread::{BinRead, BinReaderExt};
use bitflags::bitflags;
use std::fmt::Display;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::ops::DerefMut;

pub(crate) const BIG_DATA_SEGMENT_SIZE: u32 = 16344;

#[derive(BinRead)]
#[br(import(count: u32))]
pub struct KeyValueList {
    #[br(count=count)]
    pub key_value_offsets: Vec<Offset>,
}

impl From<Cell<KeyValueList>> for KeyValueList {
    fn from(cell: Cell<KeyValueList>) -> Self {
        cell.into_data()
    }
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
        #[allow(non_upper_case_globals)]
        const IS_TOMBSTONE = 0x0002;
    }
}

#[derive(BinRead)]
#[br(magic = b"vk")]
#[allow(dead_code)]
/// https://github.com/msuhanov/regf/blob/master/Windows%20registry%20file%20format%20specification.md#key-value
pub struct KeyValue {
    name_length: u16,
    data_size: u32,
    data_offset: PosValue<Offset>,
    data_type: KeyValueDataType,

    #[br(parse_with=parse_value_flags)]
    flags: KeyValueFlags,
    spare: u16,

    #[br(parse_with=parse_string, count=name_length, args(flags.contains(KeyValueFlags::VALUE_COMP_NAME)))]
    key_name_string: String,
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

pub enum RegistryValue {
    RegNone,
    RegSZ(String),
    RegExpandSZ(String),
    RegBinary(String),
    RegDWord(u32),
    RegDWordBigEndian(u32),
    RegLink(String),
    RegMultiSZ(Vec<String>),
    RegResourceList(String),
    RegFullResourceDescriptor(String),
    RegResourceRequirementsList(String),
    RegQWord(u64),
}

impl Display for RegistryValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryValue::RegNone => write!(f, "None"),
            RegistryValue::RegSZ(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegExpandSZ(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegBinary(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegDWord(val) => write!(f, "dword:0x{:08x}", val),
            RegistryValue::RegDWordBigEndian(val) => write!(f, "dword:0x{:08x}", val),
            RegistryValue::RegLink(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegMultiSZ(val) => write!(f, "\"{:?}\"", val),
            RegistryValue::RegResourceList(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegFullResourceDescriptor(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegResourceRequirementsList(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegQWord(val) => write!(f, "qword:0x{:016x}", val),
        }
    }
}

impl KeyValue {
    pub fn name(&self) -> &str {
        &self.key_name_string
    }

    pub fn value<B>(&self, hive: &mut Hive<B>) -> Result<RegistryValue>
    where
        B: BinReaderExt,
    {
        if matches!(self.data_type, KeyValueDataType::RegNone) {
            return Ok(RegistryValue::RegNone);
        }

        let data_size = self.data_size & 0x7fff_ffff;
        let raw_value =
        if self.data_size & 0x80000000 == 0x80000000 {
            hive.seek(SeekFrom::Start(self.data_offset.pos));
            let raw_data: SizedVec = hive.read_le_args((data_size as usize,))?;
            raw_data.0
        } else {
            // read Big Data
            if data_size > BIG_DATA_SEGMENT_SIZE {
                Vec::new()
            } else {
                // don't treat data as Big Data
                let data_cell: SizedVec =
                    hive.read_structure_args(self.data_offset.val, (data_size as usize,))?;
                data_cell.0
            }
        };

        Ok(RegistryValue::RegNone)
    }
}

fn parse_value_flags<R: Read + Seek>(
    reader: &mut R,
    _ro: &ReadOptions,
    _: (),
) -> BinResult<KeyValueFlags> {
    let raw_value: u16 = reader.read_le()?;
    Ok(KeyValueFlags::from_bits(raw_value).unwrap())
}

impl From<Cell<KeyValue>> for KeyValue {
    fn from(cell: Cell<KeyValue>) -> Self {
        cell.into_data()
    }
}
