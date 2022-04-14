use std::mem;
use binread::{BinRead, derive_binread};
use crate::{Cell, CellHeader};

#[derive_binread]
#[br(import(count:Option<usize>))]
pub (crate) struct CellWithU8List {
    #[br(temp)]
    header: CellHeader,

    #[br(count=count.or_else(|| Some(header.contents_size())).unwrap())]
    pub data: Vec<u8>
}

impl From<CellWithU8List> for Vec<u8> {
    fn from(cell: CellWithU8List) -> Self {
        cell.data
    }
}

#[derive(BinRead)]
#[br(import(count:usize))]
pub (crate) struct SizedVec (
    #[br(count=count)]
    pub Vec<u8>
);

impl From<Cell<SizedVec>> for SizedVec {
    fn from(cell: Cell<SizedVec>) -> Self {
        const ALIGNMENT: usize = 8;
        let _cell_expected_size = (cell.data().0.len() + mem::size_of::<i32>() + (ALIGNMENT-1)) & !(ALIGNMENT-1);
        //assert_eq!(cell.contents_size() as usize, _cell_expected_size);
        cell.into_data()
    }
}
