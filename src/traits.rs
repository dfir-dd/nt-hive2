use std::ops::{DerefMut};
use binread::{BinReaderExt};
use crate::{Offset, Hive, Result};

pub trait FromOffset<H, B>: Sized
where
    H: DerefMut<Target = Hive<B>>,
    B: BinReaderExt, {

    fn from_offset(hive: H, offset: Offset) -> Result<Self>;
}