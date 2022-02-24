use std::io::{SeekFrom};
use std::cell::RefCell;

use crate::nk;
use crate::nk::{KeyNode, KeyNodeHeader};
use crate::{NtHiveError, Result};
use binread::{BinRead, BinReaderExt, PosValue};

pub struct Hive<B>
where
    B: BinReaderExt,
{
    pub(crate) data: RefCell<B>,
    base_block: HiveBaseBlock,
    data_offset: u64,
}

impl<B> Hive<B>
where
    B: BinReaderExt,
{
    pub fn new(mut data: B) -> Result<Self> {
        data.seek(SeekFrom::Start(0))?;
        let base_block: HiveBaseBlock = data.read_le().unwrap();
        let data_offset = data.stream_position()?;

        Ok(Self {
            data: RefCell::new(data),
            base_block,
            data_offset,
        })
    }

    pub fn enum_subkeys(&self, callback: fn(&KeyNode<&Self, B>) -> Result<()>) -> Result<()> {
        let root_key_node = self.root_key_node()?;
        callback(&root_key_node)?;
        Ok(())
    }

    fn root_key_node(&self) -> Result<KeyNode<&Self, B>> {
        KeyNode::from_cell_offset(self, self.base_block.root_cell_offset)
    }

    pub fn seek_to_cell_offset(&self, data_offset: u32) -> Result<usize> {
        // Only valid data offsets are accepted here.
        assert!(data_offset != u32::MAX);

        // Accept only u32 data offsets, but convert them into usize right away for
        // slice range operations and fearless calculations.
        let data_offset = data_offset as usize;

        self.data.borrow_mut().seek(SeekFrom::Start(self.data_offset + data_offset as u64))?;
        let header: CellHeader = self.data.borrow_mut().read_le()?;

        let cell_data_offset = self.data.borrow_mut().stream_position()? as usize;

        // A cell with size > 0 is unallocated and shouldn't be processed any further by us.
        if *header.size > 0 {
            return Err(NtHiveError::UnallocatedCell {
                offset: data_offset,
                size: *header.size,
            });
        }

        Ok(cell_data_offset)
    }
}

/// this data structure follows the documentation found at
/// <https://github.com/msuhanov/regf/blob/master/Windows%20registry%20file%20format%20specification.md#format-of-primary-files>
#[allow(dead_code)]
#[derive(BinRead)]
#[br(magic = b"regf")]
struct HiveBaseBlock {
    primary_sequence_number: u32,
    secondary_sequence_number: u32,
    timestamp: u64,

    #[br(assert(major_version==1))]
    major_version: u32,

    #[br(assert(vec![3, 4, 5, 6].contains(&minor_version)))]
    minor_version: u32,

    #[br(assert(file_type==0))]
    file_type: u32,

    #[br(assert(file_format==1))]
    file_format: u32,
    root_cell_offset: u32,

    #[br(assert(data_size%4096 == 0))]
    data_size: u32,
    clustering_factor: u32,
    file_name: [u16; 32],
    #[br(count = 99)]
    padding_1: Vec<u32>,
    checksum: u32,
    #[br(count = 0x37E)]
    padding_2: Vec<u32>,
    boot_type: u32,
    boot_recover: u32,
}

#[derive(BinRead)]
#[br(magic = b"hbin")]
struct HiveBin {
    offset: u32,

    #[br(assert(size%4096 == 0))]
    size: u32,
    reserved: u64,
    timestamp: u64,
    spare: u32,
}

#[derive(BinRead)]
#[repr(packed)]
struct CellHeader {
    // The cell size must be a multiple of 8 bytes
    #[br(assert(*size%8 == 0, NtHiveError::InvalidSizeFieldAlignment{
        expected_alignment: 8,
        size: *size as usize,
        offset: size.pos as usize}))]
    size: PosValue<i32>,
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::io;

    #[test]
    fn enum_subkeys() {
        let testhive = crate::helpers::tests::testhive_vec();
        let hive = Hive::new(io::Cursor::new(testhive)).unwrap();
        assert!(hive.enum_subkeys(|_k| Ok(())).is_ok());
    }
}
