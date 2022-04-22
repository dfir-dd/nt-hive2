use std::io::{ErrorKind, Read, Seek, SeekFrom};

use crate::nk::KeyNodeWithMagic;
use crate::{nk::KeyNode, CellIterator};
use crate::Cell;
use binread::{BinRead, BinReaderExt, BinResult};

/// Represents a registry hive file.
/// 
/// Because most offsets in a registry hive file are relative to the start of the hive bins data,
/// this struct provides a own [Seek] and [Read] implementation, which can work directly
/// with those kinds of offsets. You don't know where the hive bins data starts, because [Hive] knows
/// it (this information is stored in the hive base block). To parse data from within the hive bins data,
/// use [Hive] as reader and use offsets read from the hive data structures.
/// 
/// The structure of hive files is documented at <https://github.com/msuhanov/regf/blob/master/Windows%20registry%20file%20format%20specification.md#format-of-primary-files>
pub struct Hive<B>
where
    B: BinReaderExt,
{
    data: B,
    base_block: HiveBaseBlock,
    data_offset: u32,
}

/// represents an offset (usually a 32bit value) used in registry hive files
#[derive(BinRead, Debug, Clone, Copy)]
pub struct Offset(pub u32);

impl<B> Hive<B>
where
    B: BinReaderExt,
{
    /// creates a new [Hive] object. This includes parsing the HiveBaseBlock and determining
    /// the start of the hive bins data.
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

    /// Is this really needed???
    pub fn enum_subkeys(&mut self, callback: fn(&mut Self, &KeyNode) -> BinResult<()>) -> BinResult<()> {
        let root_key_node = self.root_key_node()?;
        callback(self, &root_key_node)?;
        Ok(())
    }

    /// returns the root key of this registry hive file
    pub fn root_key_node(&mut self) -> BinResult<KeyNode> {
        let mkn: KeyNodeWithMagic = self.read_structure(self.base_block.root_cell_offset)?;
        Ok(mkn.into())
    }

    /// reads a data structure from the given offset. Read the documentation of [Cell]
    /// for a detailled discussion
    /// 
    /// # Usage
    /// 
    /// ```
    /// # use std::error::Error;
    /// # use std::fs::File;
    /// use nt_hive2::*;
    /// 
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let hive_file = File::open("tests/data/testhive")?;
    /// # let mut hive = Hive::new(hive_file)?;
    /// # let offset = hive.root_cell_offset();
    /// let my_node: KeyNodeWithMagic = hive.read_structure(offset)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_structure<T>(&mut self, offset: Offset) -> BinResult<T>
    where
        T: BinRead<Args=()> + std::convert::From<crate::Cell<T, ()>>,
    {
        log::trace!("reading cell of type {} from offset {:08x} (was: {:08x})", std::any::type_name::<T>(), offset.0 + self.data_offset, offset.0);
        
        self.seek(SeekFrom::Start(offset.0.into()))?;
        let cell: Cell<T, ()> = self.read_le().unwrap();
        assert!(cell.is_allocated());
        Ok(cell.into())
    }

    /// returns the start of the hive bins data
    pub fn data_offset(&self) -> &u32 {
        &self.data_offset
    }

    /// returns the offset of the root cell
    pub fn root_cell_offset(&self) -> Offset {
        self.base_block.root_cell_offset
    }

    pub fn into_cell_iterator<C>(self, callback: C) -> CellIterator<B, C> where C: Fn(u64) -> () {
        CellIterator::new(self, callback)
    }

    pub fn data_size(&self) -> u32 {
        self.base_block.data_size
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
