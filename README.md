> [!CAUTON]
> # `nt_hive2` is leaving github
> 
> ## Why?
>
> I believe that all men (and women, and all human between and above) are created equal. In this mindset, it does not make sense to judge people based on their birthplace, or their language, color, religion, or whatsoever.
> 
> I believe that who you are is made up of what you do. If you are caring towards other people, then that's you are. If you do harm to other people, then that's who you are.
> 
> I'm concerned of what is currently happening in the United States. I don't like it when a government thinks it is above the law. I don't like it when a government doesn't serve the people, but sees people as a threat. But that's politics.
> 
> Github is part of Microsoft, and Microsoft is supporting this government. For example, Microsoft blocked the mail accounts of ICC members because of political reasons. I don't want to get my accounts blocked or deleted arbitrarily. Therefore, I'm going to not support Microsoft in any way. That's why I'll move all my repositories away from github.
> 
> We had a good time. Cheers.
>
> ## Where?
> 
> The new place-to-be for `nt_hive2` is <https://codeberg.org/janstarke/nt-hive2>.


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
