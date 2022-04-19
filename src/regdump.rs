use nt_hive2::*;
use simplelog::{SimpleLogger, Config};
use std::fs::File;
use std::io::{Read, Seek};
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

    fn do_print_key<RS>(hive: &mut Hive<RS>, root_key: &KeyNode) -> Result<()> where RS: Read + Seek {
        let mut path = Vec::new();
        print_key(hive, &root_key, &mut path)
    }

    match File::open(&cli.hive_file) {
        Ok(data) => {
            let mut hive = Hive::new(data).unwrap();
            let root_key = &hive.root_key_node().unwrap();
            do_print_key(&mut hive, &root_key).unwrap();
        }
        Err(why) => {
            eprintln!("unable to open '{}': {}", cli.hive_file, why);
            std::process::exit(-1);
        },
    }
}

fn print_key<RS>(hive: &mut Hive<RS>, keynode: &KeyNode, path: &mut Vec<String>) -> Result<()> where RS: Read + Seek {
    path.push(keynode.name().to_string());
    println!("\n[{}]; {}", path.join("\\"), keynode.timestamp());

    print_values(keynode);

    for sk in keynode.subkeys(hive).unwrap().iter() {
        print_key(hive, &sk.borrow(), path)?;
    }
    path.pop();

    Ok(())
}

fn print_values(keynode: &KeyNode) {
    for value in keynode.values() {
        println!("\"{}\" = {}", value.name(), value.value());
    }
}