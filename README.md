# nt_hive2

> **Warning**
> The tools of this repository have been moved to <https://github.com/dfir-dd/dfir-toolkit>
>
> You can install the tools by running `cargo install dfir-toolkit`
> 
> The lib itself will stay available here


This crates aims to be a replacement of <https://github.com/ColinFinck/nt-hive>, with the following
differences:

 - use of [BinRead](https://docs.rs/binread/latest/binread/) to parse hive files
 - support of displaying last written timestamps
 - recovery of deleted cells

## Usage example for developers

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
