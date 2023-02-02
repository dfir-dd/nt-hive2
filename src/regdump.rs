use bodyfile::Bodyfile3Line;
use nt_hive2::*;
use simplelog::{SimpleLogger, Config};
use std::fs::File;
use std::io::{Read, Seek};
use std::path::PathBuf;
use anyhow::{Result, bail};
use clap::Parser;

mod logfileset;
use logfileset::LogfileSet;

#[derive(Parser)]
#[clap(name="regdump", author, version, about, long_about = None)]
struct Args {
    /// name of the file to dump
    #[arg(value_parser = validate_file)]
    pub (crate) hive_file: PathBuf,

    /// LOG1 file (name of the original hive file, together with the extension LOG1)
    #[clap(long("log1"), group("logfiles"))]
    #[arg(value_parser = validate_file)]
    log1file: Option<PathBuf>,

    /// LOG2 file (name of the original hive file, together with the extension LOG1)
    #[clap(long("log2"), group("logfiles"))]
    #[arg(value_parser = validate_file)]
    log2file: Option<PathBuf>,

    /// print as bodyfile format
    #[clap(short('b'),long("bodyfile"))]
    display_bodyfile: bool,

    /// ignore the base block (e.g. if it was encrypted by some ransomware)
    #[clap(short('I'), long)]
    ignore_base_block: bool,

    #[clap(flatten)]
    pub (crate) verbose: clap_verbosity_flag::Verbosity,
}

impl Args {
    pub fn parse_mode(&self) -> HiveParseMode {
        if self.ignore_base_block {
            match File::open(&self.hive_file) {
                Ok(data) => {
                    let hive = Hive::new(data, HiveParseMode::Raw).unwrap();
                    let offset = match hive.find_root_celloffset() {
                        Some(offset) => offset,
                        None => {
                            log::error!("scan found no root cell offset, aborting...");
                            std::process::exit(-1);
                        }
                    };
                    println!("found offset at {}", offset.0);
                    HiveParseMode::Normal(offset)
                }
                Err(why) => {
                    log::error!("unable to open '{}': {}", self.hive_file.to_string_lossy(), why);
                    std::process::exit(-1);
                },
            }
        } else {
            HiveParseMode::NormalWithBaseBlock
        }
    }

    pub fn logfileset(&self) -> Result<Option<LogfileSet>> {
        match &self.log1file {
            Some(log1file) => match &self.log2file {
                Some(log2file) => {
                    Ok(Some(LogfileSet::new(log1file, log2file)?))
                }
                None => {
                    bail!("missing LOG2 file");
                }
            }
            None => match &self.log2file {
                Some(_) => {
                    bail!("missing LOG1 file");
                }
                None => Ok(None)
            }
        }
    }

}

fn validate_file(s: &str) -> Result<PathBuf, String> {
    let pb = PathBuf::from(s);
    if pb.is_file() && pb.exists() {
        Ok(pb)
    } else {
        Err(format!("unable to read file: '{s}'"))
    }
}

fn main() -> Result<()> {
    let cli = Args::parse();
    let _ = SimpleLogger::init(cli.verbose.log_level_filter(), Config::default());

    fn do_print_key<RS>(hive: &mut Hive<RS, CleanHive>, root_key: &KeyNode, cli: &Args) -> Result<()> where RS: Read + Seek {
        let mut path = Vec::new();
        print_key(hive, root_key, &mut path, cli)
    }



    match File::open(&cli.hive_file) {
        Ok(data) => {
            let hive = Hive::new(data, cli.parse_mode()).unwrap();

            let mut clean_hive = 
            if let Some(logfileset) = cli.logfileset()? {
                logfileset.recover(hive)?
            } else {
                log::warn!("no log files provided, treating hive as if it was clean");
                hive.treat_hive_as_clean()
            };

            let root_key = &clean_hive.root_key_node().unwrap();
            do_print_key(&mut clean_hive, root_key, &cli).unwrap();
        }
        Err(why) => {
            eprintln!("unable to open '{}': {}", cli.hive_file.to_string_lossy(), why);
            std::process::exit(-1);
        },
    }
    Ok(())
}

fn print_key<RS>(hive: &mut Hive<RS, CleanHive>, keynode: &KeyNode, path: &mut Vec<String>, cli: &Args) -> Result<()> where RS: Read + Seek {
    path.push(keynode.name().to_string());

    let current_path = path.join("\\");
    if cli.display_bodyfile {
        let bf_line = Bodyfile3Line::new()
            .with_name(&current_path)
            .with_ctime(keynode.timestamp().timestamp());
        println!("{}", bf_line);
    } else {
        println!("\n[{}]; {}", &current_path, keynode.timestamp());

        print_values(keynode);
    }

    for sk in keynode.subkeys(hive).unwrap().iter() {
        print_key(hive, &sk.borrow(), path, cli)?;
    }
    path.pop();

    Ok(())
}

fn print_values(keynode: &KeyNode) {
    for value in keynode.values() {
        let data_type = match value.data_type() {
            Some(dt) => format!("{dt}:"),
            None => "".into()
        };

        println!("\"{}\" = {data_type}{}", value.name(), value.value());
    }
}