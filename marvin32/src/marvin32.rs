use std::{
    hash::Hasher,
    io::Cursor,
};

use byteorder::{LittleEndian, ReadBytesExt};

pub struct Marvin32 {
    lo: u32,
    hi: u32,
    buffer: Vec<u8>,
}

impl Marvin32 {
    pub fn new(seed: u64) -> Self {
        Self {
            lo: (seed & 0xFFFF_FFFF) as u32,
            hi: (seed >> 32) as u32,
            buffer: Vec::with_capacity(4),
        }
    }

    /// convert_to_le interprets a slice of four bytes as a little-endian u32 value.
    fn convert_to_le(p: &[u8]) -> u32 {
        let mut rdr = Cursor::new(p);
        rdr.read_u32::<LittleEndian>().unwrap()
    }

    fn const_mix(mut lo: u32, mut hi: u32, value: u32) -> (u32, u32) {
        lo = lo.wrapping_add(value);
        hi ^= lo;
        lo = lo.rotate_left(20).wrapping_add(hi);
        hi = hi.rotate_left(9) ^ lo;
        lo = lo.rotate_left(27).wrapping_add(hi);
        hi = hi.rotate_left(19);
        (lo, hi)
    }

    /// mix updates the Marvin32State with a new value `value`.
    fn mix(&mut self, value: u32) {
        (self.lo, self.hi) = Self::const_mix(self.lo, self.hi, value)
    }
}

impl Hasher for Marvin32 {
    fn finish(&self) -> u64 {
        let fin = self.buffer
            .iter()
            .rev()
            .fold(0x80u32, |fin, b| (fin << 8) | (*b as u32));

        let (lo, hi) = Self::const_mix(self.lo, self.hi, fin);
        let (lo, hi) = Self::const_mix(lo, hi, 0);
        
        (lo ^ hi).into()
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut pos = 0;
        if !self.buffer.is_empty() {
            while self.buffer.len() < 4 {
                if pos == bytes.len() {
                    return;
                }

                self.buffer.push(bytes[pos]);
                pos += 1;
            }

            self.mix(Self::convert_to_le(&self.buffer[0..4]));
            self.buffer.clear();
        }

        if bytes.len() >= 4 {
            let max_pos = bytes.len() - 4;
            while pos <= max_pos {
                self.mix(Self::convert_to_le(&bytes[pos..pos+4]));
                pos += 4;
            }
        }
        self.buffer.extend(&bytes[pos..]);
        assert!(self.buffer.len() < 4);
    }
}
