
use binread::PosValue;
use binread::{BinReaderExt, BinRead};

use crate::Hive;
use crate::Offset;
use crate::Result;
use crate::traits::FromOffset;
use crate::NtHiveError;
use std::io::{Seek, SeekFrom};
use std::ops::DerefMut;


#[derive(BinRead)]
struct CellHeader {
    // The cell size must be a multiple of 8 bytes
    #[br(assert(*size%8 == 0, NtHiveError::InvalidSizeFieldAlignment {
        expected_alignment: 8,
        size: *size as usize,
        offset: size.pos as usize}))]
    size: PosValue<i32>,
}

pub struct Cell<T>
where
    T: BinRead, {
    header: CellHeader,
    data: T,
}

impl<H, B, T> FromOffset<H, B> for Cell<T>
where
    H: DerefMut<Target = Hive<B>>,
    B: BinReaderExt,
    T: BinRead {

    fn from_offset(mut hive: H, offset: Offset) -> Result<Self> {
        let _offset = hive.seek(SeekFrom::Start(offset.0.into()))?;
        let header: CellHeader = hive.read_le()?;
        let data: T = hive.read_le()?;
        Ok(Self {
            header,
            data,
        })
    }
}

impl<T> Cell<T> where T: BinRead {
    pub fn is_deleted(&self) -> bool {
        *self.header.size > 0
    }

    pub fn contents_size(&self) -> u32 {
        (*self.header.size).abs() as u32
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
        let header: CellHeader = hive.read_le()?;
        let data: T = hive.read_le_args(args)?;
        Ok(Cell::<T> {
            header,
            data,
        })
    }
}