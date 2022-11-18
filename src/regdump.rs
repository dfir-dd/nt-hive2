use bodyfile::Bodyfile3Line;
use nt_hive2::*;
use simplelog::{SimpleLogger, Config};
use std::fs::File;
use std::io::{Read, Seek};
use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[clap(name="regdump", author, version, about, long_about = None)]
struct Args {
    /// name of the file to dump
    pub (crate) hive_file: String,

    #[clap(flatten)]
    pub (crate) verbose: clap_verbosity_flag::Verbosity,

    /// print as bodyfile format
    #[clap(short('b'),long("bodyfile"))]
    display_bodyfile: bool,

    /// ignore the base block (e.g. if it was encrypted by some ransomware)
    #[clap(short('I'), long)]
    ignore_base_block: bool,
}

fn main() {
    let cli = Args::parse();
    let _ = SimpleLogger::init(cli.verbose.log_level_filter(), Config::default());

    fn do_print_key<RS>(hive: &mut Hive<RS>, root_key: &KeyNode, cli: &Args) -> Result<()> where RS: Read + Seek {
        let mut path = Vec::new();
        print_key(hive, root_key, &mut path, cli)
    }

    let parse_mode = if cli.ignore_base_block {
        match File::open(&cli.hive_file) {
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
                log::error!("unable to open '{}': {}", cli.hive_file, why);
                std::process::exit(-1);
            },
        }
    } else {
        HiveParseMode::NormalWithBaseBlock
    };

    match File::open(&cli.hive_file) {
        Ok(data) => {
            let mut hive = Hive::new(data, parse_mode).unwrap();
            let root_key = &hive.root_key_node().unwrap();
            do_print_key(&mut hive, root_key, &cli).unwrap();
        }
        Err(why) => {
            eprintln!("unable to open '{}': {}", cli.hive_file, why);
            std::process::exit(-1);
        },
    }
}

fn print_key<RS>(hive: &mut Hive<RS>, keynode: &KeyNode, path: &mut Vec<String>, cli: &Args) -> Result<()> where RS: Read + Seek {
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
        println!("\"{}\" = {}", value.name(), value.value());
    }
}