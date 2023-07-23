# nt_hive2


> **Warning**
> This Repository has been moved to <https://github.com/janstarke/dfir-toolkit>
>
> You can install the tools by running `cargo install dfir-toolkit`
> 


This crates aims to be a replacement of <https://github.com/ColinFinck/nt-hive>, with the following
differences:

 - use of [BinRead](https://docs.rs/binread/latest/binread/) to parse hive files
 - support of displaying last written timestamps
 - recovery of deleted cells

# `regdump`

```
forensic parser library for Windows registry hive files

Usage: regdump [OPTIONS] <HIVE_FILE>

Arguments:
  <HIVE_FILE>  name of the file to dump

Options:
  -L, --log <LOGFILES>     transaction LOG file(s). This argument can be specified one or two times
  -b, --bodyfile           print as bodyfile format
  -I, --ignore-base-block  ignore the base block (e.g. if it was encrypted by some ransomware)
  -v, --verbose...         More output per occurrence
  -q, --quiet...           Less output per occurrence
  -h, --help               Print help information
  -V, --version            Print version information

```

# `hivescan`

```
scans a registry hive file for deleted entries

Usage: hivescan [OPTIONS] <HIVE_FILE>

Arguments:
  <HIVE_FILE>  name of the file to scan

Options:
  -L, --log <LOGFILES>  transaction LOG file(s). This argument can be specified one or two times
  -v, --verbose...      More output per occurrence
  -q, --quiet...        Less output per occurrence
  -b                    output as bodyfile format
  -h, --help            Print help information
  -V, --version         Print version information
```

# `cleanhive`

```
merges logfiles into a hive file

Usage: cleanhive [OPTIONS] --output <DST_HIVE> <HIVE_FILE>

Arguments:
  <HIVE_FILE>  name of the file to dump

Options:
  -L, --log <LOGFILES>     transaction LOG file(s). This argument can be specified one or two times
  -v, --verbose...         More output per occurrence
  -q, --quiet...           Less output per occurrence
  -O, --output <DST_HIVE>  name of the file to which the cleaned hive will be written
  -h, --help               Print help information
  -V, --version            Print version information
```

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
