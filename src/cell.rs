
use binread::{BinReaderExt, BinRead};

use crate::Hive;
use crate::Offset;
use crate::Result;
use crate::traits::FromOffset;
use std::io::{Seek, SeekFrom};
use std::ops::DerefMut;

pub struct Cell<T>
where
    T: BinRead, {
    size: i32,
    data: T,
}

impl<H, B, T> FromOffset<H, B> for Cell<T>
where
    H: DerefMut<Target = Hive<B>>,
    B: BinReaderExt,
    T: BinRead {

    fn from_offset(mut hive: H, offset: Offset) -> Result<Self> {
        let _offset = hive.seek(SeekFrom::Start(offset.0.into()))?;
        let size: i32 = hive.read_le()?;
        let data: T = hive.read_le()?;
        Ok(Self {
            size,
            data,
        })
    }
}

impl<T> Cell<T> where T: BinRead {
    pub fn is_deleted(&self) -> bool {
        self.size > 0
    }

    pub fn contents_size(&self) -> u32 {
        self.size.abs() as u32
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub(crate) fn into_data(self) -> T {
        self.data
    }

    pub fn from_offset_args<H, B>(mut hive: H, offset: Offset, args: T::Args) -> Result<Cell<T>> where
        H: DerefMut<Target = Hive<B>>,
        B: BinReaderExt
    {
        let _offset = hive.seek(SeekFrom::Start(offset.0.into()))?;
        let size: i32 = hive.read_le()?;
        let data: T = hive.read_le_args(args)?;
        Ok(Cell::<T> {
            size,
            data,
        })
    }
}