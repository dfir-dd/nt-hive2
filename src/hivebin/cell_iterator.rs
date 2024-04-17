use std::cell::RefCell;
use std::fmt::Debug;
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

            // we assume that we already consumed the header
            consumed_bytes: hivebin.header_size().into(),
        }
    }

    fn next_cell(&mut self) -> BinResult<Option<CellSelector>> {
        const CELL_HEADER_SIZE: usize = 4;

        // if there is not enough space in this hivebin, give up
        if self.consumed_bytes + CELL_HEADER_SIZE >= self.hivebin_size {
            return Ok(None);
        }

        let cell_offset = self.hive.borrow_mut().stream_position().unwrap();

        let header: CellHeader = self.hive.borrow_mut().read_le()?;

        let cell_size = header.size();
        let content: CellContent = self.hive.borrow_mut().read_le()?;
        self.consumed_bytes += cell_size;

        let cell_selector = CellSelector {
            offset: Offset(cell_offset.try_into().unwrap()),
            header,
            content,
        };
        self.hive.borrow_mut().seek(std::io::SeekFrom::Start(
            cell_offset + u64::try_from(cell_size).unwrap(),
        ))?;

        Ok(Some(cell_selector))
    }
}

impl<B> Iterator for CellIterator<B>
where
    B: BinReaderExt,
{
    type Item = CellSelector;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_cell() {
            Ok(v) => v,
            Err(why) => {
                if let binread::Error::Io(kind) = &why {
                    if kind.kind() != ErrorKind::UnexpectedEof {
                        log::warn!("parser error: {}", why);
                    }
                } else {
                    log::warn!("parser error: {}", why);
                }
                None
            }
        }
    }
}

#[derive(BinRead, Getters)]
#[getter(get = "pub")]
pub struct CellSelector {
    offset: Offset,
    header: CellHeader,
    content: CellContent,
}

#[derive_binread]
#[derive(Debug)]
pub enum CellContent {
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

    #[allow(clippy::upper_case_acronyms)]
    UNKNOWN,
}

#[derive(Error, Debug)]
pub enum CellLookAheadConversionError {
    #[error(
        "tried to extract some type from this cell, which is not actually stored in this cell."
    )]
    DifferentCellTypeExpected,
}

impl CellContent {
    pub fn is_nk(&self) -> bool {
        matches!(self, Self::NK(_))
    }
}

impl TryInto<KeyNode> for CellSelector {
    type Error = CellLookAheadConversionError;

    fn try_into(self) -> Result<KeyNode, Self::Error> {
        match self.content {
            CellContent::NK(nk) => Ok(nk),
            _ => Err(CellLookAheadConversionError::DifferentCellTypeExpected),
        }
    }
}
