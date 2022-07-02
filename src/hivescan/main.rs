use nt_hive2::*;
use simplelog::{SimpleLogger, Config};
use std::{fs::File};
use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

mod regtreebuilder;
mod regtreeentry;
use regtreebuilder::RegTreeBuilder;

/// scans a registry hive file for deleted entries
#[derive(Parser)]
#[clap(author, version)]
struct Args {
    /// name of the file to scan
    pub (crate) hive_file: String,

    #[clap(flatten)]
    pub (crate) verbose: clap_verbosity_flag::Verbosity,
}

fn main() {
    let cli = Args::parse();
    let _ = SimpleLogger::init(cli.verbose.log_level_filter(), Config::default());

    match File::open(&cli.hive_file) {
        Ok(data) => {
            let hive = Hive::new(data, HiveParseMode::NormalWithBaseBlock).unwrap();
            scan_hive(hive).unwrap();
        }
        Err(why) => {
            eprintln!("unable to open '{}': {}", cli.hive_file, why);
            std::process::exit(-1);
        },
    }
}

fn scan_hive(mut hive: Hive<File>) -> Result<()> {
    let progress_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>9}/{len:9}({percent}%) {msg}");
    let bar = ProgressBar::new(hive.data_size().into());
    bar.set_style(progress_style);

    bar.set_message("scanning cells");
    let builder = RegTreeBuilder::from_hive(hive, |p| bar.set_position(p));

    
    Ok(())
}