use nt_hive2::*;
use std::fs::File;
use std::io::{Read, Cursor, Seek};

fn main() {
    env_logger::init();
        
    let testhive = testhive_vec();
    let hive = Hive::new(Cursor::new(testhive)).unwrap();
    let mut path = Vec::new();
    print_key(&hive.root_key_node().unwrap(), &mut path).unwrap();
}

fn print_key<RS>(keynode: &KeyNode<&Hive<RS>, RS>, path: &mut Vec<String>) -> Result<()> where RS: Read + Seek {
    path.push(keynode.name().unwrap().to_string());
    println!("[{}]", path.join("\\"));

    print_values(keynode)?;

    for sk in keynode.subkeys().unwrap() {
        print_key(&sk, path)?;
    }
    path.pop();

    Ok(())
}

fn print_values<RS>(keynode: &KeyNode<&Hive<RS>, RS>) -> Result<()> where RS: Read + Seek {
    for value in keynode.values()? {
        println!("{}", value.name()?);
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