use crate::transcationlogs::dirtpagesref::DirtPagesRef;
use crate::transcationlogs::dirtypages::DirtPages;
use crate::transcationlogs::marvin::*;

use binread::{BinRead, BinReaderExt, BinResult};
use std::io::SeekFrom;

pub const BLOCK_SIZE: u32 = 512;
pub const HVLE_START_OFFSET: u64 = 512;
pub const HIVE_BIN_SIZE_ALIGNMENT: u32 = 4096;
#[allow(dead_code)]
pub const BASE_BLOCK_LENGTH_PRIMARY: u32 = 4096;
#[allow(dead_code)]
pub const HBIN_START_OFFSET: u64 = 600;

#[derive(Debug, Clone)]
pub struct TransactionLogs {
    pub d_pages: Vec<DirtPages>,
}

impl TransactionLogs {
    pub fn new<T: BinReaderExt>(data: &mut T, prim_sq_num: u32) -> BinResult<(Vec<Self>, u32)> {
        data.seek(SeekFrom::Start(HVLE_START_OFFSET))?;
        let mut offset = 512;
        let mut transcationlogs = Vec::new();
        let mut new_sequence_number = 0;
        let mut index = 512;

        while let Ok(log_base_block) = data.read_le::<TransactionLogsBlock>() {

            let size: u32 = log_base_block.size;
            let sequ: u32 = log_base_block.sequence_number;

            new_sequence_number = sequ;

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
            let validate_hashes = TransactionLogs::calchashes(offset, data, hash1, hash2, size)?;
          
            index += size;
            offset = index.into();
            data.seek(SeekFrom::Start(index.into()))?;
            if !validate_hashes {
                break;
            }else{
                transcationlogs.push(Self { d_pages: dirtpage });

            }
        }

        Ok((transcationlogs, new_sequence_number))
    }

    fn calchashes<T: BinReaderExt>(
        offset: u64,
        data: &mut T,
        hash1: u64,
        hash2: u64,
        size: u32,
    ) -> Result<bool, binread::Error> {
        let new_offset = offset;
        data.seek(SeekFrom::Start(new_offset + 40))?;
        let mut buff = vec![0; (size - 40) as usize];
        data.read_exact(&mut buff)?;
        let hash1_marv = Marvin32::new(0x82EF4D887A4E55C5).marvin32_hash(&buff);
        let hash1_dec = ((hash1 >> 32) ^ hash1) as u32;
        data.seek(SeekFrom::Start(new_offset))?;
        let mut buff = vec![0; (size) as usize];
        data.read_exact(&mut buff)?;
        let hash2_marv = Marvin32::new(0x82EF4D887A4E55C5).marvin32_hash(&buff[0..32]);
        let hash2_dec = ((hash2 >> 32) ^ hash2) as u32;
        if hash1_marv != hash1_dec || hash2_marv != hash2_dec {
            return Ok(false);
        } else {
            return Ok(true);
        }
      
    }

}

#[allow(dead_code)]
#[derive(BinRead, Debug, Clone, Copy)]
// if the signature isn't match then stop looping as there will be no more hvle to obtain.
#[br(magic = b"HvLE")]
pub struct TransactionLogsBlock {
    // if the block size is bigger than the size or the block size is not divided by 512 then break the loop
    #[br(assert(size > BLOCK_SIZE || size % BLOCK_SIZE == 0))]
    size: u32,
    #[br(assert(flags==0))]
    flags: u32,
    sequence_number: u32,
    //if the hbin size is bigger than the size or the hbin size is not divided by 512 then break the loop
    #[br(assert(hbin_data_size > HIVE_BIN_SIZE_ALIGNMENT || hbin_data_size % HIVE_BIN_SIZE_ALIGNMENT == 0))]
    hbin_data_size: u32,
    #[br(assert(dirty_pages_count!=0))]
    dirty_pages_count: u32,
    hash1: u64,
    hash2: u64,
}
