//! This crates aims to be a replacement of <https://github.com/ColinFinck/nt-hive>, with the following
//! differences:
//! 
//!  - use of [BinRead](https://docs.rs/binread/latest/binread/) to parse hive files
//!  - support of displaying last written timestamps
//!  - possibly recovery of deleted cells (might be added in the future)
//! 
//! # Usage example
//! 
//! ```
//! # use std::error::Error;
//! use std::fs::File;
//! use nt_hive2::*;
//! 
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let hive_file = File::open("tests/data/testhive")?;
//! let mut hive = Hive::new(hive_file, HiveParseMode::NormalWithBaseBlock)?;
//! let root_key = hive.root_key_node()?;
//! 
//! for sk in root_key.subkeys(&mut hive)?.iter() {
//!     println!("\n[{}]; last written: {}", sk.borrow().name(), sk.borrow().timestamp());
//!     for value in sk.borrow().values() {
//!         println!("\"{}\" = {}", value.name(), value.value());
//!     }
//! }
//! # Ok(())
//! # }
//! ```

mod hive;
mod hivebin;
mod util;
mod cell;
mod nk;
mod vk;
mod db;
mod subkeys_list;
mod cell_with_u8_list;
mod cell_iterator;

pub use cell::*;
pub use cell_iterator::{CellIterator, CellLookAhead, CellSelector, CellFilter};
pub use hive::{Hive, Offset, HiveParseMode};
pub use nk::{KeyNode, KeyNodeWithMagic, SubPath};
pub use vk::{KeyValue, KeyValueWithMagic, RegistryValue};