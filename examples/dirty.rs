use std::fs::File;
use std::path::PathBuf;
extern crate nt_hive2;
use log::LevelFilter;
use nt_hive2::*;
use simplelog::{SimpleLogger, Config};

fn main() -> anyhow::Result<()> {
    let _ = SimpleLogger::init(LevelFilter::Warn, Config::default());

    let mut path_logs = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path_logs.push("tests");
    path_logs.push("data");
    path_logs.push("NewDirtyHive1");

    let mut path_hive = path_logs.clone();
    let mut path_log1 = path_logs.clone();
    let mut path_log2 = path_logs.clone();
    path_hive.push("NewDirtyHive");
    path_log1.push("NewDirtyHive.LOG1");
    path_log2.push("NewDirtyHive.LOG2");

    let new_hive = format!("{}.{}", path_logs.to_string_lossy(), "clean");
    let hive_file = File::open(path_hive)?;
    let hive_log1 = File::open(path_log1)?;
    let hive_log2 = File::open(path_log2)?;
    let mut hive = 
        Hive::new(&hive_file, nt_hive2::HiveParseMode::NormalWithBaseBlock)?
        .with_transaction_log(hive_log1.try_into()?)?
        .with_transaction_log(hive_log2.try_into()?)?;

    let mut dst = File::create(new_hive)?;
    std::io::copy(&mut hive, &mut dst)?;
    Ok(())
}
