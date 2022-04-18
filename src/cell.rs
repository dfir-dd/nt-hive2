
use binread::BinRead;
use std::any::Any;


#[derive(BinRead)]
pub struct CellHeader {
    // The cell size must be a multiple of 8 bytes
    //#[br(assert(*size%8 == 0, "size {} is not divisible by 8", *size))]
    size: i32,
}

impl CellHeader {
    pub fn raw_size(&self) -> i32 {
        self.size
    }

    pub fn size(&self) -> usize {
        self.size.abs().try_into().unwrap()
    }

    pub fn contents_size(&self) -> usize {
        assert!(self.size() >= 4);
        self.size() - std::mem::size_of_val(&self.size)
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
        self.header.size > 0
    }

    pub fn is_allocated(&self) -> bool {
        self.header.size <= 0
    }

    pub fn contents_size(&self) -> u32 {
        (self.header.size).abs() as u32
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub(crate) fn into_data(self) -> T {
        self.data
    }
}