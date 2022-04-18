
use binread::{BinRead, derive_binread};
use std::{any::Any};


#[derive_binread]
pub struct CellHeader {
    // The cell size must be a multiple of 8 bytes
    #[br(temp, assert(raw_size%8 == 0, "size {} is not divisible by 8", raw_size))]
    raw_size: i32,

    #[br(calc(raw_size.abs().try_into().unwrap()))]
    size: usize,

    #[br(calc(raw_size > 0))]
    is_deleted: bool
}

impl CellHeader {
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn contents_size(&self) -> usize {
        assert!(self.size() >= 4);
        self.size() - std::mem::size_of::<i32>()
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }
}

#[derive(BinRead)]
#[br(import_tuple(data_args: A))]
pub struct Cell<T, A: Any + Copy>
where
    T: BinRead<Args=A>, {
    
    header: CellHeader,

    #[br(args_tuple(data_args))]
    data: T,
}

impl<T, A> Cell<T, A> where T: BinRead<Args=A>, A: Any + Copy {
    pub fn is_deleted(&self) -> bool {
        self.header.is_deleted
    }

    pub fn is_allocated(&self) -> bool {
        ! self.is_deleted()
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub(crate) fn into_data(self) -> T {
        self.data
    }
}