use std::{fs::File, path::Path};

use anyhow::bail;

use crate::{Hive, HiveParseMode};

pub(crate) struct TransactionLogFile {
    logfile: Hive<File>,
    primary_sequence_number: u32,
    secondary_sequence_number: u32,
}

impl TryFrom<&Path> for TransactionLogFile {
    type Error = anyhow::Error;
    fn try_from(logfile_path: &Path) -> Result<Self, Self::Error> {
        let log_file_bin = File::open(logfile_path)?;
        let logfile = Hive::new(log_file_bin, HiveParseMode::NormalWithBaseBlock)?;

        if let Some(base_block) = logfile.base_block.as_ref() {
            let primary_sequence_number = *base_block.primary_sequence_number();
            let secondary_sequence_number = *base_block.secondary_sequence_number();
            Ok(Self {
                logfile,
                primary_sequence_number,
                secondary_sequence_number,
            })
        } else {
            bail!("hive file '{}' has no base block", logfile_path.to_string_lossy());
        }
    }
}

impl Ord for TransactionLogFile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.primary_sequence_number.cmp(&other.primary_sequence_number)
    }
}

impl Eq for TransactionLogFile {
}

impl PartialOrd for TransactionLogFile {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.primary_sequence_number.partial_cmp(&other.primary_sequence_number) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        None
    }
}

impl PartialEq for TransactionLogFile {
    fn eq(&self, other: &Self) -> bool {
        self.primary_sequence_number == other.primary_sequence_number && self.secondary_sequence_number == other.secondary_sequence_number
    }
}