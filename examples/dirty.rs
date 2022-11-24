use std::fs;
use std::{ fs::File};
extern crate nt_hive2;
use nt_hive2::transcationlogs::transactionlogs::RecoverHive;
use nt_hive2::*;



fn main() {
    let path_logs = r"C:\Users\user\Desktop\000\DirtyHives\2\NewDirtyHive";
    let new_hive= format!("{}.{}",&path_logs,"clean");
    let hive_file = File::open(path_logs).unwrap();
    let  hive = Hive::new(&hive_file, nt_hive2::HiveParseMode::NormalWithBaseBlock).unwrap();
    // let dirty = RecoverHive::new().is_dirty(calc_csum);
    
    fs::write(new_hive,RecoverHive::new().recover_hive(hive, path_logs));

}
