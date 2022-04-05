use std::ops::{DerefMut};
use binread::{BinReaderExt, BinResult};
use crate::{Offset, Hive};

pub trait FromOffset<H, B>: Sized
where
    H: DerefMut<Target = Hive<B>>,
    B: BinReaderExt, {

    fn from_offset(hive: H, offset: Offset) -> BinResult<Self>;
}