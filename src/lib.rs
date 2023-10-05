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
pub mod transactionlog;
mod binary_string;

pub use cell::*;
pub use cell_iterator::*;
pub use hive::*;
pub use nk::*;
pub use vk::*;
pub use util::*;
pub use binary_string::*;