use std::io::{Seek, SeekFrom};

use binread::{BinReaderExt, BinRead, derive_binread, BinResult};

use crate::*;
use crate::hivebin::HiveBin;
use crate::subkeys_list::*;

pub struct CellIterator<B> where B: BinReaderExt {
    hive: Hive<B>,
    hivebin: Option<HiveBin>,
    read_from_hivebin: usize,
}

impl<B> CellIterator<B> where B: BinReaderExt {
    pub fn new(mut hive: Hive<B>) -> Self {
        hive.seek(SeekFrom::Start(0)).unwrap();
        Self {
            hive,
            hivebin: None,
            read_from_hivebin: 0
        }
    }
}

impl<B> Iterator for CellIterator<B> where B: BinReaderExt {
    type Item = CellSelector;

    fn next(&mut self) -> Option<Self::Item> {
        if self.hivebin.is_none() {
            let result: BinResult<HiveBin> = self.hive.read_le();
            match result {
                Err(why) => {
                    log::warn!("parser error: {}", why);
                    return None
                }
                Ok(hivebin) => {
                    self.hivebin = Some(hivebin);
                    self.read_from_hivebin = 0;
                }
            }
        }

        let start_position = self.hive.stream_position().unwrap();
        let result: BinResult<CellSelector> = self.hive.read_le();
        match result {
            Err(why) => {
                log::warn!("parser error: {}", why);
                None
            }
            Ok(selector) => {

                if self.read_from_hivebin + selector.header().size() >= self.hivebin.as_ref().unwrap().size().try_into().unwrap() {
                    // the hivebin has been completely read, the next to be read should be
                    // the next hivebin header
                    self.hivebin = None;
                }

                self.hive.seek(SeekFrom::Start(selector.header().size() as u64 +  start_position)).unwrap();
                Some(selector)
            }
        }
    }
}

#[derive(BinRead)]
pub struct CellSelector {
    header: CellHeader,
    content: CellLookAhead
}

impl CellSelector {
    pub fn header(&self) -> &CellHeader {
        &self.header
    }
    pub fn content(&self) -> &CellLookAhead {
        &self.content
    }
}

#[derive_binread]
pub enum CellLookAhead {
    #[br(magic=b"nk")] NK(KeyNode),
    #[br(magic=b"vk")] VK(KeyValue),
    #[br(magic=b"sk")] SK,
    #[br(magic=b"db")] DB,

    #[br(magic=b"li")] LI{
        #[br(temp)]
        count: u16,

        #[br(count=count)]
        items: Vec<IndexLeafItem>
    },
    #[br(magic=b"lf")] LF{
        #[br(temp)]
        count: u16,

        #[br(count=count)]
        items: Vec<FastLeafItem>
    },

    #[br(magic=b"lh")] LH{
        #[br(temp)]
        count: u16,

        #[br(count=count)]
        items: Vec<HashLeafItem>
    },
    #[br(magic=b"ri")] RI{
        #[br(temp)]
        count: u16,

        #[br(count=count)]
        items: Vec<IndexRootListElement>
    },
}

