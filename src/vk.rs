use crate::util::*;
use crate::util::SizedVec;
use crate::Cell;
use crate::Hive;
use crate::Offset;

use binread::BinResult;
use binread::PosValue;
use binread::ReadOptions;
use binread::{BinRead, BinReaderExt};
use bitflags::bitflags;
use std::fmt::Display;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

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

    /// There are also Types that do not have a value that corresponds to
    /// anything in the list above. These are typically seen in the SAM
    /// Registry hives and often correspond to part of a users SID
    /// https://binaryforay.blogspot.com/2015/01/registry-hive-basics-part-3-vk-records.html
    #[br(try)]
    data_type: Option<KeyValueDataType>,

    #[br(parse_with=parse_value_flags)]
    flags: KeyValueFlags,
    spare: u16,

    #[br(parse_with=parse_string, count=name_length, args(flags.contains(KeyValueFlags::VALUE_COMP_NAME)))]
    key_name_string: String,
}

/// Possible data types of the data belonging to a [`KeyValue`].
/// https://docs.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types
#[derive(BinRead)]
#[br(repr=u32)]
pub enum KeyValueDataType {
    /// Data with no particular type 
    RegNone = 0x0000_0000,

    /// A null-terminated string. This will be either a Unicode or an ANSI string, depending on whether you use the Unicode or ANSI functions.
    RegSZ = 0x0000_0001,

    /// A null-terminated Unicode string, containing unexpanded references to environment variables, such as "%PATH%" 
    RegExpandSZ = 0x0000_0002,

    /// Binary data in any form 
    RegBinary = 0x0000_0003,

    /// A 4-byte numerical value 
    RegDWord = 0x0000_0004,

    /// A 4-byte numerical value whose least significant byte is at the highest address 
    RegDWordBigEndian = 0x0000_0005,

    /// A Unicode string naming a symbolic link. This type is irrelevant to device and intermediate drivers 
    RegLink = 0x0000_0006,

    /// An array of null-terminated strings, terminated by another zero 
    RegMultiSZ = 0x0000_0007,

    /// A device driver's list of hardware resources, used by the driver or one of the physical devices it controls, in the \ResourceMap tree 
    RegResourceList = 0x0000_0008,

    /// A list of hardware resources that a physical device is using, detected and written into the \HardwareDescription tree by the system 
    RegFullResourceDescriptor = 0x0000_0009,

    /// A device driver's list of possible hardware resources it or one of the physical devices it controls can use, from which the system writes a subset into the \ResourceMap tree 
    RegResourceRequirementsList = 0x0000_000a,

    /// A 64-bit number.
    RegQWord = 0x0000_000b,

    /// FILETIME data
    RegFileTime = 0x0000_0010,
}

pub enum RegistryValue {
    RegNone,
    RegUnknown,
    RegSZ(String),
    RegExpandSZ(String),
    RegBinary(Vec<u8>),
    RegDWord(u32),
    RegDWordBigEndian(u32),
    RegLink(String),
    RegMultiSZ(Vec<String>),
    RegResourceList(String),
    RegFullResourceDescriptor(String),
    RegResourceRequirementsList(String),
    RegQWord(u64),
    RegFileTime,
}

impl Display for RegistryValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryValue::RegUnknown => write!(f, "Unknown"),
            RegistryValue::RegNone => write!(f, "None"),
            RegistryValue::RegSZ(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegExpandSZ(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegBinary(val) => write!(f, "{:?}", if val.len() > 16 {&val[..16]} else {val}),
            RegistryValue::RegDWord(val) => write!(f, "dword:0x{:08x}", val),
            RegistryValue::RegDWordBigEndian(val) => write!(f, "dword:0x{:08x}", val),
            RegistryValue::RegLink(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegMultiSZ(val) => write!(f, "{:?}", val),
            RegistryValue::RegResourceList(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegFullResourceDescriptor(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegResourceRequirementsList(val) => write!(f, "\"{}\"", val),
            RegistryValue::RegQWord(val) => write!(f, "qword:0x{:016x}", val),
            RegistryValue::RegFileTime => todo!(),
        }
    }
}

impl KeyValue {
    pub fn name(&self) -> &str {
        &self.key_name_string
    }

    pub fn value<B>(&self, hive: &mut Hive<B>) -> BinResult<RegistryValue>
    where
        B: BinReaderExt,
    {
        let data_type = match &self.data_type {
            None => return Ok(RegistryValue::RegUnknown),
            Some(dt) => dt
        };

        if matches!(data_type, KeyValueDataType::RegNone) {
            return Ok(RegistryValue::RegNone);
        }

        let data_size = self.data_size & 0x7fff_ffff;
        let raw_value = 
        if self.data_size & 0x80000000 == 0x80000000 {
            hive.seek(SeekFrom::Start(self.data_offset.pos))?;
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

        let result = 
        match data_type {
            KeyValueDataType::RegNone => RegistryValue::RegNone,
            KeyValueDataType::RegSZ => RegistryValue::RegSZ(parse_reg_sz(&raw_value[..])?),
            KeyValueDataType::RegExpandSZ => RegistryValue::RegExpandSZ(parse_reg_sz(&raw_value[..])?),
            KeyValueDataType::RegBinary => RegistryValue::RegBinary(raw_value),
            KeyValueDataType::RegDWord => RegistryValue::RegDWord(Cursor::new(raw_value).read_le()?),
            KeyValueDataType::RegDWordBigEndian => RegistryValue::RegDWordBigEndian(Cursor::new(raw_value).read_be()?),
            KeyValueDataType::RegLink => RegistryValue::RegNone,
            KeyValueDataType::RegMultiSZ => RegistryValue::RegMultiSZ(parse_reg_multi_sz(&raw_value[..])?),
            KeyValueDataType::RegResourceList => RegistryValue::RegNone,
            KeyValueDataType::RegFullResourceDescriptor => RegistryValue::RegNone,
            KeyValueDataType::RegResourceRequirementsList => RegistryValue::RegNone,
            KeyValueDataType::RegQWord => RegistryValue::RegQWord(Cursor::new(raw_value).read_le()?),
            KeyValueDataType::RegFileTime => RegistryValue::RegFileTime,
        };

        Ok(result)
    }
}

fn parse_value_flags<R: Read + Seek>(
    reader: &mut R,
    _ro: &ReadOptions,
    _: (),
) -> BinResult<KeyValueFlags> {
    let raw_value: u16 = reader.read_le()?;
    Ok(KeyValueFlags::from_bits_truncate(raw_value))
}

impl From<Cell<KeyValue>> for KeyValue {
    fn from(cell: Cell<KeyValue>) -> Self {
        cell.into_data()
    }
}
