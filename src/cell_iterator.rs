use std::io::{Seek, SeekFrom, ErrorKind};

use binread::{BinReaderExt, BinRead, derive_binread, BinResult};

use crate::*;
use crate::hivebin::HiveBin;
use crate::subkeys_list::*;

pub struct CellIterator<B, C> where B: BinReaderExt, C: Fn(u64) -> () {
    hive: Hive<B>,
    hivebin: Option<HiveBin>,
    read_from_hivebin: usize,
    callback: C,
}

impl<B, C> CellIterator<B, C> where B: BinReaderExt, C: Fn(u64) -> () {
    pub fn new(mut hive: Hive<B>, callback: C) -> Self {
        hive.seek(SeekFrom::Start(0)).unwrap();
        Self {
            hive,
            hivebin: None,
            read_from_hivebin: 0,
            callback
        }
    }

    fn read_hivebin_header(&mut self) -> BinResult<()> {
        let result: BinResult<HiveBin> = self.hive.read_le();
        match result {
            Err(why) => {
                if let binread::Error::Io(kind) = &why {
                    if kind.kind() == ErrorKind::UnexpectedEof {
                        return Err(why);
                    }
                }
                log::warn!("parser error: {}", why);
                Err(why)
            }
            Ok(hivebin) => {
                self.hivebin = Some(hivebin);
                self.read_from_hivebin = 0;
                Ok(())
            }
        }
    }
}

impl<B, C> Iterator for CellIterator<B, C> where B: BinReaderExt, C: Fn(u64) -> () {
    type Item = CellSelector;

    fn next(&mut self) -> Option<Self::Item> {
        if self.hivebin.is_none() {
            if self.read_hivebin_header().is_err() {
                return None;
            }
        }

        let start_position = self.hive.stream_position().unwrap();
        
        // there might be the start of a nw hive bin at this position
        if start_position & (! 0xfff) == start_position {
            log::trace!("trying to read at {:08x}", start_position + 4096);
            let result: BinResult<HiveBin> = self.hive.read_le();
            if let Ok(hivebin) = result {
                self.hivebin = Some(hivebin);
                self.read_from_hivebin = 0;

                log::trace!("found a new hivebin here");
            }

            (self.callback)(self.hive.stream_position().unwrap());
        }


        let start_position = self.hive.stream_position().unwrap();
        log::trace!("reading a cell at {:08x}", start_position + 4096);
        let result: BinResult<CellSelector> = self.hive.read_le();
        match result {
            Err(why) => {
                if let binread::Error::Io(kind) = &why {
                    if kind.kind() == ErrorKind::UnexpectedEof {
                        return None;
                    }
                }
                log::warn!("parser error: {}", why);
                (self.callback)(self.hive.stream_position().unwrap());
                None
            }

            Ok(selector) => {

                if self.read_from_hivebin + selector.header().size() >= self.hivebin.as_ref().unwrap().size().try_into().unwrap() {
                    // the hivebin has been completely read, the next to be read should be
                    // the next hivebin header
                    self.hivebin = None;
                }

                log::trace!("skipping {} bytes to {:08x}", selector.header().size(), start_position as usize + selector.header().size());

                self.hive.seek(SeekFrom::Start(selector.header().size() as u64 +  start_position)).unwrap();
                (self.callback)(self.hive.stream_position().unwrap());
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
    UNKNOWN
}

