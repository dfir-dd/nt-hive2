use nt_hive2::*;
use std::fs::File;
use std::io::{Read, Cursor, Seek};

fn main() {
    env_logger::init();
        
    let testhive = testhive_vec();
    let mut hive = Hive::new(Cursor::new(testhive)).unwrap();
    let mut path = Vec::new();
    let root_key = &hive.root_key_node().unwrap();
    print_key(&mut hive, &root_key, &mut path).unwrap();
}

fn print_key<RS>(hive: &mut Hive<RS>, keynode: &KeyNode, path: &mut Vec<String>) -> Result<()> where RS: Read + Seek {
    path.push(keynode.name().to_string());
    println!("[{}]; {}", path.join("\\"), keynode.timestamp());

    print_values(hive, keynode)?;

    for sk in keynode.subkeys(hive).unwrap() {
        print_key(hive, &sk, path)?;
    }
    path.pop();

    Ok(())
}

fn print_values<RS>(hive: &mut Hive<RS>, keynode: &KeyNode) -> Result<()> where RS: Read + Seek {
    for value in keynode.values(hive)? {
        println!("{} = {}", value.name(), value.value(hive)?);
    }
    Ok(())
}


pub fn testhive_vec() -> Vec<u8> {
    let mut buffer = Vec::new();
    File::open("tests/data/testhive")
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();
    buffer
}