use binread::{BinRead, BinReaderExt, BinResult};

pub const BASE_BLOCK_LENGTH_PRIMARY: u32 = 4096;

#[derive(BinRead,Debug, Clone)]
pub struct DirtPagesRef {
    pub offset: u32,
    pub size: u32,
}

impl DirtPagesRef {
    /// this function is read the offset of the primary have and also locate the offset on the current logs
    pub fn read_dirt<T: BinReaderExt>(
        stream: &mut T,
        drtpagecnt: u32,
    ) -> BinResult<Vec<DirtPagesRef>> {
        let mut i = 0;
        let mut vec = Vec::new();
        //loop through the number of dirty page count refrence and the grap the offset and page size
        while i < drtpagecnt {
            let offset: u32 =  stream.read_le()?;

            let page_size: u32 =  stream.read_le()?;
            

            // you have to add 4096 to the offset as the
            let offsetx = offset + BASE_BLOCK_LENGTH_PRIMARY;
            i += 1;
            // push it on vec to used later for recovery
            vec.push(DirtPagesRef {
                offset: offsetx,
                size: page_size,
            })
        }
        Ok(vec)
    }
}
