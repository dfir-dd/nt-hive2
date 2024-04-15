mod cell_iterator;

use std::{cell::RefCell, ops::Deref, rc::Rc};
pub use cell_iterator::*;

use binread::{derive_binread, BinReaderExt};
use getset::Getters;

use crate::{CleanHive, Hive, Offset};

#[derive_binread]
#[derive(Getters)]
#[br(magic = b"hbin")]
#[allow(dead_code)]
#[getset(get = "pub")]
pub struct _HiveBin {
    // Offset of a current hive bin in bytes, relative from the start of the
    // hive bins data
    offset: Offset,

    // Size of a current hive bin in bytes
    size: u32,

    reserved: u64,

    // FILETIME (UTC), defined for the first hive bin only (see below)
    //
    // A Timestamp in the header of the first hive bin acts as a backup copy of
    // a Last written timestamp in the base block.
    timestamp: u64,

    // This field has no meaning on a disk (see below)
    //
    // The Spare field is used when shifting hive bins and cells in memory. In
    // Windows 2000, the same field is called MemAlloc, it is used to track
    // memory allocations for hive bins.
    spare: u32,
}

pub struct HiveBin<B>
where
    B: BinReaderExt,
{
    hive: Rc<RefCell<Hive<B, CleanHive>>>,
    hivebin: _HiveBin,
}

impl<B> HiveBin<B>
where
    B: BinReaderExt,
{
    pub fn new(hive: Rc<RefCell<Hive<B, CleanHive>>>) -> anyhow::Result<Self> {
        let hivebin = hive.borrow_mut().read_le()?;
        Ok(Self { hive, hivebin })
    }

    pub fn cells(&self) -> impl Iterator<Item = CellSelector> {
        CellIterator::new(self, Rc::clone(&self.hive))
    }

    pub fn header_size(&self) -> u8 {
        32
    }
}

impl<B> Deref for HiveBin<B>
where
    B: BinReaderExt,
{
    type Target = _HiveBin;

    fn deref(&self) -> &Self::Target {
        &self.hivebin
    }
}
