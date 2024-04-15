use std::cell::RefCell;
use std::io::{ErrorKind, Seek};
use std::rc::Rc;

use binread::{derive_binread, BinRead, BinReaderExt, BinResult};
use derive_getters::Getters;
use thiserror::Error;

use crate::hivebin::HiveBin;
use crate::subkeys_list::*;
use crate::*;

pub struct CellIterator<B>
where
    B: BinReaderExt,
{
    hive: Rc<RefCell<Hive<B, CleanHive>>>,
    hivebin_size: usize,
    consumed_bytes: usize,
}

impl<B> CellIterator<B>
where
    B: BinReaderExt,
{
    pub fn new(hivebin: &HiveBin<B>, hive: Rc<RefCell<Hive<B, CleanHive>>>) -> Self {
        Self {
            hive,
            hivebin_size: (*hivebin.size()).try_into().unwrap(),
            consumed_bytes: hivebin.header_size().into(),
        }
    }

    fn parse<T: BinRead>(&self) -> Option<T> {
        let r: BinResult<T> = self.hive.borrow_mut().read_le();
        match r {
            Ok(t) => Some(t),
            Err(why) => {
                if let binread::Error::Io(kind) = &why {
                    if kind.kind() != ErrorKind::UnexpectedEof {
                        log::warn!("parser error: {}", why);
                    }
                }
                None
            }
        }
    }
}

impl<B> Iterator for CellIterator<B>
where
    B: BinReaderExt,
{
    type Item = CellSelector;

    fn next(&mut self) -> Option<Self::Item> {
        const CELL_HEADER_SIZE: usize = 4;

        // if there is not enough space in this hivebin, give up
        if self.consumed_bytes + CELL_HEADER_SIZE >= self.hivebin_size {
            return None;
        }

        let cell_offset = self.hive.borrow_mut().stream_position().unwrap();

        if let Some(header) = self.parse::<CellHeader>() {
            if let Some(lookahead) = self.parse::<CellLookAhead>() {
                self.consumed_bytes += header.size();
                return Some(CellSelector {
                    offset: Offset(cell_offset.try_into().unwrap()),
                    header,
                    content: lookahead,
                });
            }
        }

        None
    }
}

#[derive(BinRead, Getters)]
#[getter(get = "pub")]
pub struct CellSelector {
    offset: Offset,
    header: CellHeader,
    content: CellLookAhead,
}

#[derive_binread]
pub enum CellLookAhead {
    #[br(magic = b"nk")]
    NK(KeyNode),
    #[br(magic = b"vk")]
    VK(KeyValue),
    #[br(magic = b"sk")]
    SK,
    #[br(magic = b"db")]
    DB,

    #[br(magic = b"li")]
    LI {
        #[br(temp)]
        count: u16,

        #[br(count=count)]
        items: Vec<IndexLeafItem>,
    },
    #[br(magic = b"lf")]
    LF {
        #[br(temp)]
        count: u16,

        #[br(count=count)]
        items: Vec<FastLeafItem>,
    },

    #[br(magic = b"lh")]
    LH {
        #[br(temp)]
        count: u16,

        #[br(count=count)]
        items: Vec<HashLeafItem>,
    },
    #[br(magic = b"ri")]
    RI {
        #[br(temp)]
        count: u16,

        #[br(count=count)]
        items: Vec<IndexRootListElement>,
    },
    UNKNOWN,
}

#[derive(Error, Debug)]
pub enum CellLookAheadConversionError {
    #[error(
        "tried to extract some type from this cell, which is not actually stored in this cell."
    )]
    DifferentCellTypeExpected,
}

impl CellLookAhead {
    pub fn is_nk(&self) -> bool {
        matches!(self, Self::NK(_))
    }
}

impl TryInto<KeyNode> for CellSelector {
    type Error = CellLookAheadConversionError;

    fn try_into(self) -> Result<KeyNode, Self::Error> {
        match self.content {
            CellLookAhead::NK(nk) => Ok(nk),
            _ => Err(CellLookAheadConversionError::DifferentCellTypeExpected),
        }
    }
}
