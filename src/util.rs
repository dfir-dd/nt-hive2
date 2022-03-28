use std::io::{Read, Seek};

use binread::{ReadOptions, BinResult, BinRead, BinReaderExt};
use chrono::{DateTime, Utc};
use encoding_rs::{ISO_8859_15, UTF_16LE};
use winstructs::timestamp::WinTimestamp;

use crate::NtHiveError;


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
            err: Box::new(NtHiveError::StringEncodingError)})
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