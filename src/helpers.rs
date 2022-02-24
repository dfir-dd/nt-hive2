// Copyright 2019-2021 Colin Finck <colin@reactos.org>
// SPDX-License-Identifier: GPL-2.0-or-later

use core::ops::Range;

macro_rules! iter_try {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => return Some(Err(e)),
        }
    };
}

#[cfg(test)]
pub mod tests {
    use std::fs::File;
    use std::io::Read;

    pub fn testhive_vec() -> Vec<u8> {
        let mut buffer = Vec::new();
        File::open("tests/data/testhive")
            .unwrap()
            .read_to_end(&mut buffer)
            .unwrap();
        buffer
    }
}
