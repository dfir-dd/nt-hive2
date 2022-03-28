use std::{io::{Read, Seek}, mem};

use binread::{ReadOptions, BinResult, BinRead, BinReaderExt};
use chrono::{DateTime, Utc};
use encoding_rs::{ISO_8859_15, UTF_16LE};
use winstructs::timestamp::WinTimestamp;

use crate::{Cell};


pub (crate) fn parse_string<R: Read + Seek>(reader: &mut R, ro: &ReadOptions, params: (bool,))
-> BinResult<String> {
    let raw_string = Vec::<u8>::read_options(reader, ro, ())?;

    let (cow, _, had_errors) = 
    if params.0 {
        ISO_8859_15.decode(&raw_string[..])
    } else {
        UTF_16LE.decode(&raw_string[..])
    };

    if had_errors {
        Err(binread::error::Error::Custom { pos: ro.offset, err: Box::new(format!("unable to decode String as offset 0x{:08x}", ro.offset))})
    } else {
        Ok(cow.to_string())
    }
}

pub (crate) fn parse_timestamp<R: Read + Seek>(reader: &mut R, _ro: &ReadOptions, _: ())
-> BinResult<DateTime<Utc>> {
    let raw_timestamp: [u8;8] = reader.read_le()?;
    let timestamp = WinTimestamp::new(&raw_timestamp).unwrap();
    Ok(timestamp.to_datetime())
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
        let cell_expected_size = (cell.data().0.len() + mem::size_of::<i32>() + (ALIGNMENT-1)) & !(ALIGNMENT-1);
        assert_eq!(cell.contents_size() as usize, cell_expected_size);
        cell.into_data()
    }
}
