use crate::transcationlogs::dirtpagesref::DirtPagesRef;
use crate::transcationlogs::dirtypages::DirtPages;
use crate::transcationlogs::marvin::*;

use binread::{BinRead, BinReaderExt, BinResult};
use std::io::SeekFrom;

pub const BLOCK_SIZE: u32 = 512;
pub const HVLE_START_OFFSET: u64 = 512;
pub const HIVE_BIN_SIZE_ALIGNMENT: u32 = 4096;
pub const BASE_BLOCK_LENGTH_PRIMARY: u32 = 4096;
pub const HBIN_START_OFFSET: u64 = 600;

#[derive(Debug, Clone)]
pub struct TransctionLogs {
    pub d_pages: Vec<DirtPages>,
}

impl TransctionLogs {
    pub fn new<T: BinReaderExt>(data: &mut T, prim_sq_num: u32) -> BinResult<(Vec<Self>, u32)> {
        data.seek(SeekFrom::Start(HVLE_START_OFFSET))?;
        let mut offset = 512;
        let mut transcationlogs = Vec::new();
        let mut count = 0;
        let mut new_sequence_number = 0;
        let mut index = 512;

        loop {
            let log_base_block: TransctionLogsBlock = match data.read_le() {
                Ok(n) => n,
                Err(e) => break,
            };

            let size: u32 = log_base_block.size;
            let flag: u32 = log_base_block.flags;
            let sequ: u32 = log_base_block.sequence_number;

            new_sequence_number = sequ;

            let hbindatasize: u32 = log_base_block.hbin_data_size;
            let drtpagecnt = log_base_block.dirty_pages_count;

            // get hash1 and hash2 each is 8 bytes
            let hash1: u64 = log_base_block.hash1;
            let hash2: u64 = log_base_block.hash2;

            // i have set the size and offset to public >> this need to be fixed later
            let dirtpagesref = match DirtPagesRef::read_dirt(data, drtpagecnt) {
                Ok(n) => n,
                Err(e) => panic!("{:?}", e),
            };

            let dirtpage = match DirtPages::read_dirt_pages(data, &dirtpagesref) {
                Ok(n) => n,
                Err(e) => panic!("{:?}", e),
            };
            //calc the hashes and validate them
            let new_offset = offset;
            data.seek(SeekFrom::Start(new_offset + 40))?;
            let mut buff = vec![0; (size - 40) as usize];
            data.read_exact(&mut buff)?;
            let hash1_calc = hash(0x82EF4D887A4E55C5, &buff);
            let hash1_dec = ((hash1 >> 32) ^ hash1) as u32;
            data.seek(SeekFrom::Start(new_offset))?;
            let mut buff = vec![0; (size) as usize];
            data.read_exact(&mut buff)?;
            let hash2_calc = hash(0x82EF4D887A4E55C5, &buff[0..32]);
            let hash2_dec = ((hash2 >> 32) ^ hash2) as u32;
            if hash1_calc != hash1_dec || hash2_calc != hash2_dec {
                // emm i'm still unsure if this is the right way to do it but i tried :)
                // break;
            };
            transcationlogs.push(Self {
                d_pages: dirtpage,
            });
            count = count + 1;
            index = index + size;

            data.seek(SeekFrom::Start(index.into()))?;
        }

        Ok((transcationlogs, new_sequence_number))
    }
}

#[allow(dead_code)]
#[derive(BinRead, Debug, Clone, Copy)]
// if the signture isn't match then stop looping as there will be no more hvle to obtain.
#[br(magic = b"HvLE")]
pub struct TransctionLogsBlock {
    // if the block size is bigger than the size or the block size is not dvidied by 512 then break the loop
    #[br(assert(size > BLOCK_SIZE || size % BLOCK_SIZE == 0))]
    size: u32,
    #[br(assert(flags==0))]
    flags: u32,
    sequence_number: u32,
    //if the hbin size is bigger than the size or the hbin size is not dvidied by 512 then break the loop
    #[br(assert(hbin_data_size > HIVE_BIN_SIZE_ALIGNMENT || hbin_data_size % HIVE_BIN_SIZE_ALIGNMENT == 0))]
    hbin_data_size: u32,
    #[br(assert(dirty_pages_count!=0))]
    dirty_pages_count: u32,
    hash1: u64,
    hash2: u64,
}
