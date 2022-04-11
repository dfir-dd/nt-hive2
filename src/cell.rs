
use binread::PosValue;
use binread::{BinReaderExt, BinRead, BinResult};

use crate::Hive;
use crate::Offset;
use crate::traits::FromOffset;
use std::io::{Seek, SeekFrom};
use std::ops::DerefMut;


#[derive(BinRead)]
pub struct CellHeader {
    // The cell size must be a multiple of 8 bytes
    #[br(assert(*size%8 == 0))]
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

    fn from_offset(mut hive: H, offset: Offset) -> BinResult<Self> {
        let _offset = hive.seek(SeekFrom::Start(offset.0.into())).unwrap();
        log::debug!("seeked to 0x{:08x} (0x{:08x})", _offset, _offset + (*hive.data_offset() as u64));
        let header: CellHeader = hive.read_le().unwrap();

        let pos = hive.stream_position()?;
        log::debug!("current position is 0x{:08x} (0x{:08x})", pos, pos + (*hive.data_offset() as u64));
        let data: T = hive.read_le().unwrap();
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

    pub fn is_allocated(&self) -> bool {
        *self.header.size <= 0
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

    pub fn from_offset_args<H, B>(mut hive: H, offset: Offset, args: T::Args) -> BinResult<Cell<T>> where
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