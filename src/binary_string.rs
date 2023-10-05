use std::io::{Read, Seek};

use binread::{BinRead, BinResult, ReadOptions};
use encoding_rs::{UTF_16LE, WINDOWS_1252};

pub struct ShouldBeUtf16(Vec<u8>);

impl From<Vec<u8>> for ShouldBeUtf16 {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}
pub struct ShouldBeAscii(Vec<u8>);
impl From<Vec<u8>> for ShouldBeAscii {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum BinaryString {
    Clean(String),
    Tainted(String),
}

impl From<String> for BinaryString {
    fn from(value: String) -> Self {
        Self::Clean(value)
    }
}

impl From<ShouldBeUtf16> for BinaryString {
    fn from(value: ShouldBeUtf16) -> Self {
        let (cow, _, had_errors) = UTF_16LE.decode(&value.0[..]);
        if had_errors {
            log::error!(
                "error while decoding bytes {raw_string:?} into UTF-16 string",
                raw_string = &value.0[..]
            );
            Self::Tainted(cow.to_string())
        } else {
            Self::Clean(cow.to_string())
        }
    }
}

impl From<ShouldBeAscii> for BinaryString {
    fn from(value: ShouldBeAscii) -> Self {
        let (cow, _, had_errors) = WINDOWS_1252.decode(&value.0[..]);
        if had_errors {
            log::error!(
                "error while decoding bytes {raw_string:?} into ASCII string",
                raw_string = &value.0[..]
            );
            Self::Tainted(cow.to_string())
        } else {
            Self::Clean(cow.to_string())
        }
    }
}

impl BinaryString {
    pub fn parse<R: Read + Seek>(
        reader: &mut R,
        ro: &ReadOptions,
        params: (bool,),
    ) -> BinResult<Self> {
        let raw_string = Vec::<u8>::read_options(reader, ro, ())?;

        Ok(if params.0 {
            ShouldBeAscii(raw_string).into()
        } else {
            ShouldBeUtf16(raw_string).into()
        })
    }
}
