use nt_hive2::*;
use simplelog::{SimpleLogger, Config};
use std::{fs::File};
use anyhow::{Result, bail};
use clap::Parser;

mod regtreebuilder;
mod regtreeentry;
mod hivescanapplication;
use hivescanapplication::*;


fn main() -> Result<()> {
    let mut cli = Args::parse();
    let _ = SimpleLogger::init(cli.verbose.log_level_filter(), Config::default());

    match File::open(&cli.hive_file) {
        Ok(data) => {
            let hive = Hive::new(data, HiveParseMode::NormalWithBaseBlock).unwrap();

            let clean_hive = 
            match cli.logfiles.len() {
                0 => {
                    log::warn!("no log files provided, treating hive as if it was clean");
                    hive.treat_hive_as_clean()
                }
                1 => {
                    hive.with_transaction_log(File::open(cli.logfiles.pop().unwrap())?.try_into()?)?
                    .apply_logs()
                }
                2 => {
                    hive.with_transaction_log(File::open(cli.logfiles.pop().unwrap())?.try_into()?)?
                    .with_transaction_log(File::open(cli.logfiles.pop().unwrap())?.try_into()?)?
                    .apply_logs()
                }
                _ => {
                    bail!("more than two transaction log files are not supported")
                }
            };
            
            let mut app = HiveScanApplication::new(cli, clean_hive);
            app.run()

        }
        Err(why) => {
            eprintln!("unable to open '{}': {}", cli.hive_file, why);
            std::process::exit(-1);
        },
    }
}