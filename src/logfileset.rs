use std::{fs::File, path::PathBuf};

use binread::BinReaderExt;
use nt_hive2::{Hive, CleanHive, DirtyHive, ContainsHive};

pub(crate) struct LogfileSet {
    log1file: File,
    log2file: File,
}

impl LogfileSet {
    pub fn new(log1file: &PathBuf, log2file: &PathBuf) -> Result<Self, std::io::Error> {
        let log1file = File::open(log1file)?;
        let log2file = File::open(log2file)?;
        Ok(Self { log1file, log2file })
    }

    pub fn recover<B>(self, hive: Hive<B, DirtyHive>) -> anyhow::Result<Hive<B, CleanHive>>
    where
        B: BinReaderExt,
    {
        Ok(hive
            .with_transaction_log(self.log1file.try_into()?)?
            .with_transaction_log(self.log2file.try_into()?)?
            .apply_logs())
    }
}
