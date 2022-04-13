use binread::{derive_binread, ReadOptions, BinResult, BinReaderExt, FilePtr32};
use std::io::{SeekFrom, Read, Seek};

use crate::{Offset, CellHeader, sized_vec::SizedVec};

pub const BIGDATA_MAX_SEGMENT_SIZE: u16 = 16344;

#[derive_binread]
#[br(magic = b"db")]
pub struct BigData {
    #[br(temp)]
    segments_count: u16,

    #[br(temp, deref_now, args(segments_count,))]
    segments: FilePtr32<BigDataOffsetList>,

    #[br(parse_with=obtain_data_bytes, args(&segments,))]
    pub bytes: Vec<u8>
}

#[derive_binread]
#[br(import(count:u16))]
struct BigDataOffsetList {
    #[br(temp)]
    header: CellHeader,

    #[br(count=count)]
    pub segments: Vec<Offset>
}

fn obtain_data_bytes<R: Read + Seek>(
    reader: &mut R,
    _ro: &ReadOptions,
    args: (&FilePtr32<BigDataOffsetList>,),
) -> BinResult<Vec<u8>> {
    let offsets_ptr = args.0;

    match offsets_ptr.value {
        None => Ok(Vec::new()),
        Some(ref offset_list) => {
            log::debug!("found {} offsets at 0x{:08x}:", offset_list.segments.len(), offsets_ptr.ptr + 4096);
            for offset in &offset_list.segments {
                log::debug!("  0x{:08x}", offset.0 as usize + 4096);
            }

            // allocate the maximum expected size of data
            let mut res = Vec::with_capacity(offset_list.segments.len() * BIGDATA_MAX_SEGMENT_SIZE as usize);
        
            for offset in &offset_list.segments {
                log::debug!("reading data pointed to at 0x{:08x}", offset.0 as usize + 4096);
                let _offset = reader.seek(SeekFrom::Start(offset.0.into()))?;
                let header: CellHeader = reader.read_le()?;
                let data: SizedVec = reader.read_le_args((header.contents_size(),))?;
                res.extend(data.0);
            }
        
            // possibly we have allocated too much space; freeing it now
            res.shrink_to_fit();
            Ok(res)
        }
    }
}