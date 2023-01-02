use std::{io::{Read, Seek}, fs::File};

use binread::{BinRead, ReadOptions, BinResult, BinReaderExt};
use derive_getters::Getters;

use crate::hive::HiveBaseBlock;

use self::transactionlogsentry::TransactionLogsEntry;

mod transactionlogsentry;
//mod dirtpagesref;
//mod dirtypages;
mod dirty_pages;
mod transactionlogfile;
//pub mod transactionlogs;

pub use dirty_pages::*;

// <https://github.com/msuhanov/regf/blob/master/Windows%20registry%20file%20format%20specification.md#new-format>
#[derive(BinRead, Debug, Clone, Default, Getters)]
pub struct TransactionLog {

    /// A modified partial backup copy of a base block is stored in the first
    /// sector of a transaction log file in the same way as in the old format
    /// and for the same purpose. However, the File type field is set to 6.
    #[br(args(6,))]
    base_block: HiveBaseBlock,

    #[br(assert(!log_entries.is_empty()))]
    log_entries: Vec<TransactionLogsEntry>
}


fn read_log_entries<R: Read + Seek>(
    reader: &mut R,
    ro: &ReadOptions,
    params: (),
) -> BinResult<Vec<TransactionLogsEntry>> {
    let mut log_entries = Vec::new();

    // read until an error occurs
    loop {
        if let Ok(entry) = reader.read_le::<TransactionLogsEntry>() {
            log_entries.push(entry);
        } else {
            return Ok(log_entries);
        }
    }
}

impl From<TransactionLog> for Vec<TransactionLogsEntry> {
    fn from(log: TransactionLog) -> Self {
        log.log_entries
    }
}

impl TryFrom<File> for TransactionLog {
    type Error = binread::Error;

    fn try_from(mut file: File) -> Result<Self, Self::Error> {
        file.read_le::<TransactionLog>()
    }

}