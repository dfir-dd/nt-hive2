use std::{fs::File, path::PathBuf};

use nt_hive2::{transactionlog::TransactionLog, CleanHive, ContainsHive, Hive, BASEBLOCK_SIZE};

#[test]
fn test_cleanhive_plain() {
    let mut data_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    data_path.push("tests");
    data_path.push("data");
    data_path.push("NewDirtyHive1");

    let mut hive_path = data_path.clone();
    let mut log1_path = data_path.clone();
    let mut log2_path = data_path.clone();

    hive_path.push("NewDirtyHive");
    log1_path.push("NewDirtyHive.LOG1");
    log2_path.push("NewDirtyHive.LOG2");

    let mut hive = Hive::new(
        File::open(&hive_path).unwrap(),
        nt_hive2::HiveParseMode::NormalWithBaseBlock,
    )
    .unwrap()
    .treat_hive_as_clean();

    assert!(hive.is_checksum_valid().unwrap());
}

#[test]
fn test_cleanhive_with_log1() {
    let mut data_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    data_path.push("tests");
    data_path.push("data");
    data_path.push("NewDirtyHive1");

    let mut hive_path = data_path.clone();
    let mut log1_path = data_path.clone();
    let mut log2_path = data_path.clone();

    hive_path.push("NewDirtyHive");
    log1_path.push("NewDirtyHive.LOG1");
    log2_path.push("NewDirtyHive.LOG2");

    let mut hive = Hive::new(
        File::open(&hive_path).unwrap(),
        nt_hive2::HiveParseMode::NormalWithBaseBlock,
    )
    .unwrap()
    .with_transaction_log(TransactionLog::try_from(File::open(&log1_path).unwrap()).unwrap())
    .unwrap()
    .apply_logs();

    assert!(hive.is_checksum_valid().unwrap());
}

#[test]
fn test_cleanhive_with_log1_and_log2() {
    let mut data_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    data_path.push("tests");
    data_path.push("data");
    data_path.push("NewDirtyHive1");

    let mut hive_path = data_path.clone();
    let mut log1_path = data_path.clone();
    let mut log2_path = data_path.clone();

    hive_path.push("NewDirtyHive");
    log1_path.push("NewDirtyHive.LOG1");
    log2_path.push("NewDirtyHive.LOG2");

    let mut hive = Hive::new(
        File::open(&hive_path).unwrap(),
        nt_hive2::HiveParseMode::NormalWithBaseBlock,
    )
    .unwrap()
    .with_transaction_log(TransactionLog::try_from(File::open(&log1_path).unwrap()).unwrap())
    .unwrap()
    .with_transaction_log(TransactionLog::try_from(File::open(&log2_path).unwrap()).unwrap())
    .unwrap()
    .apply_logs();

    assert!(hive.is_checksum_valid().unwrap());

    let mut buffer = vec![0; 1024*1024];
    std::io::copy(&mut hive, &mut buffer).unwrap();
    assert!(Hive::<File, CleanHive>::validate_checksum(&buffer[0..BASEBLOCK_SIZE]).is_ok());

}
