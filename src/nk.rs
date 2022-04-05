use std::io::Read;
use std::io::Seek;

use crate::Cell;
use crate::Hive;
use crate::subkeys_list::*;
use crate::Offset;
use crate::vk::KeyValueList;
use crate::vk::KeyValue;
use binread::BinResult;
use binread::ReadOptions;
use binread::{BinRead, BinReaderExt};
use bitflags::bitflags;
use chrono::DateTime;
use chrono::Utc;
use crate::util::{parse_string, parse_timestamp};

#[allow(dead_code)]
#[derive(BinRead)]
#[br(magic = b"nk")]
pub struct KeyNode {
    #[br(parse_with=parse_node_flags)]
    flags: KeyNodeFlags,
    
    #[br(parse_with=parse_timestamp)]
    timestamp: DateTime<Utc>,
    access_bits: u32,
    parent: u32,
    subkey_count: u32,
    volatile_subkey_count: u32,
    subkeys_list_offset: Offset,
    volatile_subkeys_list_offset: Offset,
    key_values_count: u32,
    key_values_list_offset: Offset,
    key_security_offset: Offset,
    class_name_offset: Offset,
    max_subkey_name: u32,
    max_subkey_class_name: u32,
    max_value_name: u32,
    max_value_data: u32,
    work_var: u32,
    key_name_length: u16,
    class_name_length: u16,

    #[br(parse_with=parse_string, count=key_name_length, args(flags.contains(KeyNodeFlags::KEY_COMP_NAME)))]
    key_name_string: String,
}

fn parse_node_flags<R: Read + Seek>(reader: &mut R, _ro: &ReadOptions, _: ())
-> BinResult<KeyNodeFlags>
{
    let raw_value: u16 = reader.read_le()?;
    Ok(KeyNodeFlags::from_bits(raw_value).unwrap())
}

bitflags! {
    struct KeyNodeFlags: u16 {
        /// This is a volatile key (not stored on disk).
        const KEY_IS_VOLATILE = 0x0001;
        /// This is the mount point of another hive (not stored on disk).
        const KEY_HIVE_EXIT = 0x0002;
        /// This is the root key.
        const KEY_HIVE_ENTRY = 0x0004;
        /// This key cannot be deleted.
        const KEY_NO_DELETE = 0x0008;
        /// This key is a symbolic link.
        const KEY_SYM_LINK = 0x0010;
        /// The key name is in (extended) ASCII instead of UTF-16LE.
        const KEY_COMP_NAME = 0x0020;
        /// This key is a predefined handle.
        const KEY_PREDEF_HANDLE = 0x0040;
        /// This key was virtualized at least once.
        const KEY_VIRT_MIRRORED = 0x0080;
        /// This is a virtual key.
        const KEY_VIRT_TARGET = 0x0100;
        /// This key is part of a virtual store path.
        const KEY_VIRTUAL_STORE = 0x0200;
    }
}

impl KeyNode
{
    /// Returns the name of this Key Node.
    pub fn name(&self) -> &str {
        &self.key_name_string
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub fn subkeys<'a, B>(&'a self, hive: &mut Hive<B>) -> BinResult<Vec<Self>> where B: BinReaderExt{
        let offset = self.subkeys_list_offset;

        if offset.0 == u32::MAX{
            return Ok(Vec::new());
        }

        let subkeys_list: SubKeysList = hive.read_structure(offset)?;

        log::debug!("SubKeyList is of type '{}'", match subkeys_list {
            SubKeysList::IndexLeaf { items: _, ..} => "IndexLeaf",
            SubKeysList::FastLeaf { items: _, ..} => "FastLeaf",
            SubKeysList::HashLeaf { items: _, ..} => "HashLeaf",
            SubKeysList::IndexRoot { items: _, ..} => "IndexRoot",
        });

        log::debug!("{:?}", subkeys_list);

        if subkeys_list.is_index_root() {
            log::debug!("reading indirect subkey lists");
            let subkeys: BinResult<Vec<_>>= subkeys_list.into_offsets().map(|o| {
                let subsubkeys_list: SubKeysList = hive.read_structure(o)?;
                assert!(!subsubkeys_list.is_index_root());

                let subkeys: BinResult<Vec<_>> = subsubkeys_list.into_offsets().map(|o2| {
                    let nk: KeyNode = hive.read_structure(o2)?;
                    Ok(nk)
                }).collect();
                subkeys
            }).collect();

            match subkeys {
                Err(why) => Err(why),
                Ok(sk) => Ok(sk.into_iter().flatten().collect())
            }
        } else {
            log::debug!("reading single subkey list");
            let subkeys: BinResult<Vec<_>> = subkeys_list.into_offsets().map(|offset| {
                let nk: KeyNode = hive.read_structure(offset)?;
                Ok(nk)
            }).collect();
            subkeys
        }
    }

    /// returns a list of all values of this very Key Node
    pub fn values<B>(&self, hive: &mut Hive<B>) -> BinResult<Vec<KeyValue>> where B: BinReaderExt {
        let mut result = Vec::with_capacity(self.key_values_count as usize);
        if self.key_values_count > 0 && self.key_values_list_offset.0 != u32::MAX {
            let kv_list: KeyValueList = hive.read_structure_args(self.key_values_list_offset, (self.key_values_count,)).unwrap();

            for offset in kv_list.key_value_offsets.iter() {
                log::debug!("reading KeyValue from {} (0x{:08x})", offset.0 + hive.data_offset(), offset.0 + hive.data_offset());
                result.push(hive.read_structure(*offset).unwrap());
            }
        }
        Ok(result)
    }
}

impl From<Cell<KeyNode>> for KeyNode {
    fn from(cell: Cell<KeyNode>) -> Self {
        cell.into_data()
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::io;

    #[test]
    fn enum_subkeys() {
        let testhive = crate::helpers::tests::testhive_vec();
        let mut hive = Hive::new(io::Cursor::new(testhive)).unwrap();
        assert!(hive.enum_subkeys(|hive, k: &KeyNode| {
            assert_eq!(k.name(), "ROOT");

            for sk in k.subkeys(hive).unwrap() {
                println!("{}", sk.name());
            }

            Ok(())
        }).is_ok());
    }
}

