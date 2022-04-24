use nt_hive2::*;
use simplelog::{SimpleLogger, Config};
use std::fs::File;
use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

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

fn scan_hive(hive: Hive<File>) -> Result<()> {
    let progress_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>9}/{len:9}({percent}%) {msg}");
    let bar = ProgressBar::new(hive.data_size().into());
    bar.set_style(progress_style);

    bar.set_message("scanning cells");
    
    let iterator = hive
        .into_cell_iterator(|p| bar.set_position(p))
        .with_filter(CellFilter::DeletedOnly);

    let mut count_nk = Counter::default();
    let mut count_vk = Counter::default();

    for cell in iterator {
        let cell_type = 
        match cell.content() {
            CellLookAhead::NK(_nk) => "nk",
            CellLookAhead::VK(_vk) => "vk",
            _ => continue,
            /*
            CellLookAhead::SK => todo!(),
            CellLookAhead::DB => todo!(),
            CellLookAhead::LI { count, items } => todo!(),
            CellLookAhead::LF { count, items } => todo!(),
            CellLookAhead::LH { count, items } => todo!(),
            CellLookAhead::RI { count, items } => todo!(),
            */
        };
        if cell.header().is_deleted() {
            println!("{}:0x{:08x}", cell_type, cell.offset().0);
        }
        match cell.content() {
            CellLookAhead::NK(_nk) => {
                if cell.header().is_deleted() {
                    count_nk.deleted += 1;
                } else {
                    count_nk.allocated += 1;
                }
            }
            CellLookAhead::VK(_vk) => {
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
    Ok(())
}