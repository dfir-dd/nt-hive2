use std::{io, num::TryFromIntError};
use displaydoc::Display;

pub type Result<T, E = binread::Error> = core::result::Result<T, E>;
