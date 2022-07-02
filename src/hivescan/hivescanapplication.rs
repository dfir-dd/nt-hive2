use nt_hive2::*;
use crate::regtreeentry::RegTreeEntry;
use std::{fs::File};
use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use crate::regtreebuilder::RegTreeBuilder;

/// scans a registry hive file for deleted entries
#[derive(Parser)]
#[clap(author, version)]
pub (crate) struct Args {
    /// name of the file to scan
    pub (crate) hive_file: String,

    #[clap(flatten)]
    pub (crate) verbose: clap_verbosity_flag::Verbosity,
}

pub (crate) struct HiveScanApplication{

    #[allow(dead_code)]
    cli: Args,
    
    data_offset: u32,
    hive: Option<Hive<File>>
}

impl HiveScanApplication {
    pub fn new(cli: Args, hive: Hive<File>) -> Self {
        Self { cli, data_offset: *hive.data_offset(), hive: Some(hive) }
    }

    pub fn run(&mut self) -> Result<()> {
        assert!(self.hive.is_some());

        let progress_style = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>9}/{len:9}({percent}%) {msg}");
        let bar = ProgressBar::new(self.hive.as_ref().unwrap().data_size().into());
        bar.set_style(progress_style);

        bar.set_message("scanning cells");
        let builder = RegTreeBuilder::from_hive(self.hive.take().unwrap(), |p| bar.set_position(p));
        
        assert!(self.hive.is_none());
        
        for node in builder.root_nodes() {
            self.print_entry("", &node.borrow());
        }
        Ok(())
    }

    fn print_entry(&self, path: &str, entry: &RegTreeEntry) {
        let path = format!("{}/{}", path, entry.nk().name());
    
        if entry.is_deleted() {
            println!("[{}]; found at offset 0x{:x}", path, entry.offset().0 + self.data_offset);
        }
    
        for child in entry.children() {
            self.print_entry(&path, &child);
        }
    }
}
