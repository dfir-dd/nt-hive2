use nt_hive2::*;
use simplelog::{SimpleLogger, Config};
use std::fs::File;
use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// name of the file to dump
    pub (crate) hive_file: String,

    #[clap(flatten)]
    pub (crate) verbose: clap_verbosity_flag::Verbosity,
}

fn main() {
    let cli = Args::parse();
    let _ = SimpleLogger::init(cli.verbose.log_level_filter(), Config::default());

    match File::open(&cli.hive_file) {
        Ok(data) => {
            let hive = Hive::new(data).unwrap();
            scan_hive(hive).unwrap();
        }
        Err(why) => {
            eprintln!("unable to open '{}': {}", cli.hive_file, why);
            std::process::exit(-1);
        },
    }
}

#[derive(Default)]
struct Counter {
    pub deleted: usize,
    pub allocated: usize
}

fn scan_hive(mut hive: Hive<File>) -> Result<()> {
    let iterator = hive.into_cell_iterator();

    let mut count_nk = Counter::default();
    let mut count_vk = Counter::default();

    for cell in iterator {
        match cell.content() {
            CellLookAhead::NK(nk) => {
                if cell.header().is_deleted() {
                    count_nk.deleted += 1;
                } else {
                    count_nk.allocated += 1;
                }
            }
            CellLookAhead::VK(vk) => {
                if cell.header().is_deleted() {
                    count_vk.deleted += 1;
                } else {
                    count_vk.allocated += 1;
                }
            }
            _ => ()
            /*
            CellLookAhead::SK => todo!(),
            CellLookAhead::DB => todo!(),
            CellLookAhead::LI { count, items } => todo!(),
            CellLookAhead::LF { count, items } => todo!(),
            CellLookAhead::LH { count, items } => todo!(),
            CellLookAhead::RI { count, items } => todo!(),
            */
        }
    }

    println!("found {} deleted and {} allocated KeyNodes", count_nk.deleted, count_nk.allocated);
    println!("found {} deleted and {} allocated KeyValues", count_vk.deleted, count_vk.allocated);
    Ok(())
}