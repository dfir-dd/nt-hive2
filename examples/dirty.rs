use std::fs;
use std::fs::File;
use std::path::PathBuf;
extern crate nt_hive2;
use anyhow::anyhow;
use nt_hive2::transcationlogs::transactionlogs::RecoverHive;
use nt_hive2::*;

fn main() -> anyhow::Result<()> {
    let mut path_logs = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path_logs.push("tests");
    path_logs.push("data");
    path_logs.push("NewDirtyHive1");
    path_logs.push("NewDirtyHive");
    let new_hive = format!("{}.{}", path_logs.to_string_lossy(), "clean");
    let hive_file = File::open(path_logs.clone()).unwrap();
    let hive = Hive::new(&hive_file, nt_hive2::HiveParseMode::NormalWithBaseBlock).unwrap();
    // let dirty = RecoverHive::new().is_dirty(calc_csum);

    fs::write(
        new_hive,
        RecoverHive::default().recover_hive(hive, &path_logs.to_string_lossy()),
    )
    .map_err(|why| anyhow!(why))
}
