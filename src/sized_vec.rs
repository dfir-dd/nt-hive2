use std::mem;
use binread::BinRead;
use crate::Cell;


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