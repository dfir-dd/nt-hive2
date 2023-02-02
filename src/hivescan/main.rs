use nt_hive2::*;
use simplelog::{SimpleLogger, Config};
use std::{fs::File};
use anyhow::Result;
use clap::Parser;

mod regtreebuilder;
mod regtreeentry;
mod hivescanapplication;
use hivescanapplication::*;


fn main() -> Result<()> {
    let cli = Args::parse();
    let _ = SimpleLogger::init(cli.verbose.log_level_filter(), Config::default());

    match File::open(&cli.hive_file) {
        Ok(data) => {
            let hive = Hive::new(data, HiveParseMode::NormalWithBaseBlock).unwrap();
            
            let mut app = HiveScanApplication::new(cli, hive);
            app.run()

        }
        Err(why) => {
            eprintln!("unable to open '{}': {}", cli.hive_file, why);
            std::process::exit(-1);
        },
    }
}