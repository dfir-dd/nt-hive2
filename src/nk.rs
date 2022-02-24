use std::ops::Deref;

use crate::Hive;
use crate::Result;
use binread::{BinRead, BinReaderExt};

#[allow(dead_code)]
#[derive(BinRead)]
#[repr(packed)]
#[br(magic = b"nk")]
pub(crate) struct KeyNodeHeader {
    flags: u16,
    timestamp: u64,
    spare: u32,
    parent: u32,
    subkey_count: u32,
    volatile_subkey_count: u32,
    subkeys_list_offset: u32,
    volatile_subkeys_list_offset: u32,
    key_values_count: u32,
    key_values_list_offset: u32,
    key_security_offset: u32,
    class_name_offset: u32,
    max_subkey_name: u32,
    max_subkey_class_name: u32,
    max_value_name: u32,
    max_value_data: u32,
    work_var: u32,
    key_name_length: u16,
    class_name_length: u16,

    #[br(count=key_name_length)]
    key_name_string: Vec<u8>,
}

pub struct KeyNode<H, B>
where
    H: Deref<Target = Hive<B>>,
    B: BinReaderExt,
{
    header: KeyNodeHeader,
    hive: H,
}

impl<H, B> KeyNode<H, B>
where
    H: Deref<Target = Hive<B>>,
    B: BinReaderExt,
{
    pub fn from_cell_offset(hive: H, offset: u32) -> Result<Self> {
        hive.seek_to_cell_offset(offset);
        let header: KeyNodeHeader = hive.data.borrow_mut().read_le().unwrap();
        Ok(Self { header, hive })
    }
}
