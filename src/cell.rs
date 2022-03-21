
use binread::{BinReaderExt, BinRead};

use crate::Hive;
use crate::Offset;
use crate::Result;
use crate::traits::FromHiveBinOffset;
use std::ops::Deref;

pub struct Cell<T>
where
    T: BinRead, {
    size: i32,
    data: T,
}

impl<H, B, T> FromHiveBinOffset<H, B> for Cell<T>
where
    H: Deref<Target = Hive<B>> + Copy,
    B: BinReaderExt,
    T: BinRead {
    fn from_hive_bin_offset(hive: H, offset: Offset) -> Result<Self> {
        let offset = hive.seek_to_offset(offset)?;
        let size: i32 = hive.data.borrow_mut().read_le()?;
        let data: T = hive.data.borrow_mut().read_le()?;
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

    pub fn into_data(self) -> T {
        self.data
    }
}