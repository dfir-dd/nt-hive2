use binread::{BinReaderExt, BinResult};
use crate::transcationlogs::dirtpagesref::*;

#[derive(Debug, Clone)]
pub struct DirtPages {
    pub data: Vec<u8>,
    pub primary_offset: u32,
    pub page_size: u32,
}

impl DirtPages {
    ///this function is to start recovering the transaction logs based on the offset and the data size obtained from the dirty page reference
    pub fn read_dirt_pages<T: BinReaderExt>(
        stream: &mut T,
        drtpageref: &Vec<DirtPagesRef>,
    ) -> BinResult<Vec<DirtPages>> {
        let mut vec = Vec::new();
        //loop through the dirty page references and extract the date and then past them into the primary hive for recovery purposes
        for dirtypageref in drtpageref {
            let primary_offset = dirtypageref.offset;
            let page_size = dirtypageref.size;
            let mut data = vec![0; page_size as usize];
            //obtain the data based on the length of the page size
            stream.read_exact(data.as_mut_slice())?;
            //replace the obtain data in to the primary hive
            vec.push(DirtPages {
                data,
                primary_offset,
                page_size,
            })
        }
        Ok(vec)
    }
}