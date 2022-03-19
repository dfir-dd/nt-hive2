use std::ops::Deref;
use binread::BinReaderExt;
use crate::{Offset, Hive, Result};

pub trait FromHiveBinOffset<H, B>: Sized
where
    H: Deref<Target = Hive<B>> + Copy,
    B: BinReaderExt, {

    fn from_hive_bin_offset(hive: H, offset: Offset) -> Result<Self>;
}