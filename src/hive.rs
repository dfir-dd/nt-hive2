use std::io::{ErrorKind, Read, Seek, SeekFrom};

use crate::nk::KeyNode;
use crate::traits::FromOffset;
use crate::{Cell};
use binread::{BinRead, BinReaderExt, BinResult};

pub struct Hive<B>
where
    B: BinReaderExt,
{
    data: B,
    base_block: HiveBaseBlock,
    data_offset: u32,
}

#[derive(BinRead, Debug, Clone, Copy)]
pub struct Offset(pub u32);

impl<B> Hive<B>
where
    B: BinReaderExt,
{
    pub fn new(mut data: B) -> BinResult<Self> {
        data.seek(SeekFrom::Start(0))?;
        let base_block: HiveBaseBlock = data.read_le().unwrap();
        let data_offset = data.stream_position()? as u32;

        Ok(Self {
            data,
            base_block,
            data_offset,
        })
    }

    pub fn enum_subkeys(&mut self, callback: fn(&mut Self, &KeyNode) -> BinResult<()>) -> BinResult<()> {
        let root_key_node = self.root_key_node()?;
        callback(self, &root_key_node)?;
        Ok(())
    }

    pub fn root_key_node(&mut self) -> BinResult<KeyNode> {
        self.read_structure(self.base_block.root_cell_offset)
    }

    
    pub fn read_structure<T>(&mut self, offset: Offset) -> BinResult<T>
    where
        T: BinRead<Args=()> + std::convert::From<crate::Cell<T, ()>>,
    {
        log::debug!("reading cell of type {} from offset {:08x} (was: {:08x})", std::any::type_name::<T>(), offset.0 + self.data_offset, offset.0);
        self.seek(SeekFrom::Start(offset.0.into()))?;
        let cell: Cell<T, ()> = self.read_le().unwrap();
        assert!(cell.is_allocated());
        Ok(cell.into())
    }
    /*
    pub fn read_structure_args<T>(&mut self, offset: Offset, args: T::Args) -> BinResult<T>
    where
        T: BinRead<Args=()> + std::convert::From<crate::Cell<T>>
    {
        let cell: Cell<T> = Cell::from_offset_args(self, offset, args)?;
        assert!(cell.is_allocated());
        Ok(cell.into())
    }
    */

    pub fn data_offset(&self) -> &u32 {
        &self.data_offset
    }
}

impl<B> Read for Hive<B>
where
    B: BinReaderExt,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.data.read(buf)
    }
}

impl<B> Seek for Hive<B>
where
    B: BinReaderExt,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_offset = match pos {
            SeekFrom::Start(dst) =>
                self
                .data
                .seek(SeekFrom::Start(dst + self.data_offset as u64))?,
            SeekFrom::End(_) => self.data.seek(pos)?,
            SeekFrom::Current(_) => self.data.seek(pos)?,
        };
        if new_offset < self.data_offset as u64 {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("tried seek to invalid offset: {:?}", pos),
            ));
        }
        Ok(new_offset - self.data_offset as u64)
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
    root_cell_offset: Offset,

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

#[cfg(test)]
mod tests {
    use crate::*;
    use std::io;

    #[test]
    fn load_hive() {
        let testhive = crate::helpers::tests::testhive_vec();
        let mut hive = Hive::new(io::Cursor::new(testhive)).unwrap();
        assert!(hive
            .enum_subkeys(|_, k| {
                assert_eq!(k.name(), "ROOT");
                Ok(())
            })
            .is_ok());
    }
}
