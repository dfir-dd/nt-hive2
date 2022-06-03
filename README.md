# nt_hive2

This crates aims to be a replacement of <https://github.com/ColinFinck/nt-hive>, with the following
differences:

 - use of [BinRead](https://docs.rs/binread/latest/binread/) to parse hive files
 - support of displaying last written timestamps
 - possibly recovery of deleted cells (might be added in the future)

# `regdump`

```
regdump 2.1.0
forensic parser library for Windows registry hive files

USAGE:
    regdump [OPTIONS] <HIVE_FILE>

ARGS:
    <HIVE_FILE>    name of the file to dump

OPTIONS:
    -b, --bodyfile             print as bodyfile format
    -h, --help                 Print help information
    -I, --ignore-base-block    ignore the base block (e.g. if it was encrypted by some ransomware)
    -q, --quiet                Less output per occurrence
    -v, --verbose              More output per occurrence
    -V, --version              Print version information
```

## Usage example

```rust
use std::fs::File;
use nt_hive2::*;

#
let hive_file = File::open("tests/data/testhive")?;
let mut hive = Hive::new(hive_file)?;
let root_key = hive.root_key_node()?;

for sk in root_key.subkeys(&mut hive)?.iter() {
    println!("\n[{}]; last written: {}", sk.borrow().name(), sk.borrow().timestamp());
    for value in sk.borrow().values() {
        println!("\"{}\" = {}", value.name(), value.value());
    }
}
```

License: GPL-3.0
