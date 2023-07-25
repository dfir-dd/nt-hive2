use std::fs::File;
use std::path::PathBuf;
extern crate nt_hive2;
use crate::nt_hive2::{BaseBlock, ContainsHive};
use binwrite::BinWrite;
use log::LevelFilter;
use nt_hive2::*;
use simplelog::{Config, SimpleLogger};

macro_rules! path {
    ($base: expr, $($part:expr), *) => ({
        let mut x = PathBuf::from($base);
        $(
            x.push($part);
        )*
        x
    })
}

fn main() -> anyhow::Result<()> {
    let _ = SimpleLogger::init(LevelFilter::Trace, Config::default());

    let path_clean = path!(
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "data",
        "NewDirtyHive1.clean"
    );
    let path_hive = path!(
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "data",
        "NewDirtyHive1",
        "NewDirtyHive"
    );
    let path_log1 = path!(
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "data",
        "NewDirtyHive1",
        "NewDirtyHive.LOG1"
    );
    let path_log2 = path!(
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "data",
        "NewDirtyHive1",
        "NewDirtyHive.LOG2"
    );

    let hive_file = File::open(path_hive)?;
    let hive_log1 = File::open(path_log1)?;
    let hive_log2 = File::open(path_log2)?;
    let mut hive = Hive::new(&hive_file, nt_hive2::HiveParseMode::NormalWithBaseBlock)?
        .with_transaction_log(hive_log1.try_into()?)?
        .with_transaction_log(hive_log2.try_into()?)?
        .apply_logs();
    //.treat_hive_as_clean();

    let mut dst = File::create(path_clean)?;
    let base_block = hive.base_block().unwrap();
    log::info!("write with checksum 0x{:08x}", base_block.checksum());
    log::info!(
        "write with offset 0x{:08x}",
        base_block.root_cell_offset().0
    );
    log::info!("write with data size 0x{:08x}", base_block.data_size());
    base_block.write(&mut dst)?;
    std::io::copy(&mut hive, &mut dst)?;
    Ok(())
}
