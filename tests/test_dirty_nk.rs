use std::{io::{Cursor, Read, Seek, SeekFrom}, rc::Rc, cell::RefCell};

use binread::{BinReaderExt, BinRead, derive_binread, ReadOptions, BinResult, FilePtr32};
use chrono::{DateTime, Utc};
use nt_hive2::{KeyNodeFlags, Offset, Cell, KeyValue, KeyValueList, KeyValueCell, KeyValueWithMagic, BinaryString};
use winstructs::timestamp::WinTimestamp;

#[test]
fn test_dirty_nk() {
    let test_data = vec![
        0x6e, 0x6b, 0x00, 0x00, 0x43, 0xc7, 0x10, 0xcd, 0x53, 0x42, 0xd9, 0x01, 0x02, 0x00, 0x00,
        0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x00, 0x00, 0x28, 0x8c, 0x35, 0x00, 0x38,
        0x02, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x00,
        0x00, 0x40, 0x0d, 0xeb, 0xdc, 0x00, 0x66, 0x00, 0x88, 0x90, 0xf4, 0x3a, 0xb3, 0x12, 0x02,
        0xd9, 0x01, 0xe8, 0xff, 0xff, 0xff, 0x76, 0x6b, 0x00, 0x00, 0x1a, 0x00, 0x00, 0x00, 0x08,
        0x8c, 0x35, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xe0, 0xff, 0xff, 0xff,
        0x63, 0x00, 0x61, 0x00, 0x5f, 0x00, 0x61, 0x00, 0x75, 0x00, 0x74, 0x00, 0x6f, 0x00, 0x5f,
        0x00, 0x66, 0x00, 0x69, 0x00, 0x6c, 0x00, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0xff,
        0xff, 0xff, 0xf0, 0x8b, 0x35, 0x00, 0xa0, 0xff, 0xff, 0xff, 0x6e, 0x6b, 0x00, 0x00, 0x12,
        0xa0, 0x10, 0xcd, 0x53, 0x42, 0xd9, 0x01, 0x02, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0x01, 0x00, 0x00, 0x00, 0xc8, 0x8c, 0x35, 0x00, 0x38, 0x02, 0x00, 0x00, 0xff, 0xff,
        0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1a,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x41, 0x0d, 0xea, 0xdc,
        0x00, 0x65, 0x00, 0x88, 0xd0, 0xd2, 0x5f, 0xb3, 0x12, 0x02, 0x69, 0x00,
    ];
    let mut reader = Cursor::new(test_data);
    let nk: KeyNodeWithMagic = reader.read_le().unwrap();
    assert_eq!(nk.0.key_name_string, BinaryString::Tainted("ീ�昀蠀\u{f490}댺Ȓ".to_owned()));
    //assert_eq!(nk.0.key_name_string, "ീ�昀蠀\u{f490}댺Ȓ");
}


#[derive(BinRead)]
#[br(magic = b"nk")]
pub struct KeyNodeWithMagic(KeyNode);

/// represents a registry key node (as documented in <https://github.com/msuhanov/regf/blob/master/Windows%20registry%20file%20format%20specification.md#key-node>)
#[allow(dead_code)]
#[derive_binread]
pub struct KeyNode {
    #[br(parse_with=parse_node_flags)]
    pub(crate) flags: KeyNodeFlags,

    #[br(parse_with=parse_timestamp)]
    timestamp: DateTime<Utc>,
    access_bits: u32,
    pub parent: Offset,
    subkey_count: u32,

    #[br(temp)]
    volatile_subkey_count: u32,
    subkeys_list_offset: Offset,

    #[br(temp)]
    volatile_subkeys_list_offset: Offset,

    #[br(temp)]
    key_values_count: u32,

    /*#[br(   if(key_values_count > 0),
            deref_now,
            restore_position,
            args(key_values_count as usize))]
    key_values_list: Option<FilePtr32<Cell<KeyValueList, (usize,)>>>,
    key_values_list: Offset,
    */

    #[br(temp)]
    key_values_list_offset: u32,

    #[br(temp)]
    key_security_offset: Offset,

    #[br(temp)]
    class_name_offset: Offset,

    #[br(temp)]
    max_subkey_name: u32,

    #[br(temp)]
    max_subkey_class_name: u32,

    #[br(temp)]
    max_value_name: u32,

    #[br(temp)]
    max_value_data: u32,

    #[br(temp)]
    work_var: u32,

    #[br(temp)]
    key_name_length: u16,

    #[br(temp)]
    class_name_length: u16,

    #[br(   parse_with=BinaryString::parse,
            count=key_name_length,
            args(flags.contains(KeyNodeFlags::KEY_COMP_NAME)))]
    key_name_string: BinaryString,

    /*
    #[br(   if(key_values_count > 0 && key_values_list_offset != u32::MAX),
            parse_with=read_values,
            args(key_values_list.as_ref(), ))]
    values: Vec<KeyValue>,

    #[br(default)]
    subkeys: Rc<RefCell<Vec<Rc<RefCell<Self>>>>>,
    */
}

fn parse_node_flags<R: Read + Seek>(
    reader: &mut R,
    _ro: &ReadOptions,
    _: (),
) -> BinResult<KeyNodeFlags> {
    let raw_value: u16 = reader.read_le()?;
    Ok(KeyNodeFlags::from_bits(raw_value).unwrap())
}

fn parse_timestamp<R: Read + Seek>(reader: &mut R, _ro: &ReadOptions, _: ())
-> BinResult<DateTime<Utc>> {
    let raw_timestamp: [u8;8] = reader.read_le()?;
    let timestamp = WinTimestamp::new(&raw_timestamp).unwrap();
    Ok(timestamp.to_datetime())
}

fn read_values<R: Read + Seek>(
    reader: &mut R,
    _ro: &ReadOptions,
    args: (Option<&FilePtr32<KeyValueCell>>,),
) -> BinResult<Vec<KeyValue>> {
    Ok(match args.0 {
        None => Vec::new(),
        Some(key_values_list) => match &key_values_list.value {
            None => Vec::new(),
            Some(kv_list_cell) => {
                let kv_list: &KeyValueList = kv_list_cell.data();
                let mut result = Vec::with_capacity(kv_list.key_value_offsets.len());
                for offset in kv_list.key_value_offsets.iter() {
                    reader.seek(SeekFrom::Start(offset.0.into()))?;
                    let vk_result: BinResult<Cell<KeyValueWithMagic, ()>> = reader.read_le();
                    match vk_result {
                        Ok(vk) => result.push(vk.into()),
                        Err(why) => {
                            log::debug!("error while parsing KeyValue: {}", why);
                        }
                    }
                }
                result
            }
        },
    })
}