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
        Err(binread::error::Error::Custom {
            pos: ro.offset,
            err: Box::new(format!("unable to decode String at offset 0x{:08x}", ro.offset))})
    } else {
        Ok(cow.to_string())
    }
}

pub (crate) fn parse_reg_sz(raw_string: &[u8]) -> BinResult<String> {
    let (cow, _, had_errors) = UTF_16LE.decode(raw_string);
    if ! had_errors {
        assert_eq!(raw_string.len(), cow.len()*2);
        return Ok(cow.to_string());
    } else {

        let (cow, _, had_errors) = ISO_8859_15.decode(raw_string);
        if had_errors {
            Err(binread::error::Error::Custom {
                pos: 0,
                err: Box::new("unable to decode RegSZ string")})
        } else {
            assert_eq!(raw_string.len(), cow.len());
            Ok(cow.to_string())
        }
    }
}

pub (crate) fn parse_reg_multi_sz(raw_string: &[u8]) -> BinResult<Vec<String>> {
    let multi_string: Vec<String> = parse_reg_sz(raw_string)?.split('\0')
        .map(|x| x.to_owned())
        .collect();
    Ok(multi_string)
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
