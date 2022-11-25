use std::convert::TryInto;

// this code copied from "https://github.com/dgryski/marvin32-rs/blob/d1c6a60f407542562097e49d30f2bbb11948b137/src/lib.rs"
// big thanks to dgrski for saving my time and effort
struct State {
    lo: u32,
    hi: u32,
}

impl State {
    pub fn update(&mut self, v: u32) {
        self.lo = self.lo.wrapping_add(v);
        self.hi ^= self.lo;
        self.lo = self.lo.rotate_left(20).wrapping_add(self.hi);
        self.hi = self.hi.rotate_left(9) ^ self.lo;
        self.lo = self.lo.rotate_left(27).wrapping_add(self.hi);
        self.hi = self.hi.rotate_left(19);
    }
}

pub fn hash(seed: u64, data: &[u8]) -> u32 {
    let mut bytes = data;

    let mut s = State {
        lo: seed as u32,
        hi: (seed >> 32) as u32,
    };

    while bytes.len() >= 4 {
        let k1 = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        s.update(k1);
        bytes = &bytes[4..];
    }

    let fin = match bytes.len() {
        0 => 0x80,
        1 => 0x80_u32 << 8 | bytes[0] as u32,
        2 => 0x80_u32 << 16 | u16::from_le_bytes(bytes[..2].try_into().unwrap()) as u32,
        3 => {
            0x80_u32 << 24
                | u16::from_le_bytes(bytes[..2].try_into().unwrap()) as u32
                | (bytes[2] as u32) << 16
        }
        _ => panic!("len > 3"),
    };

    s.update(fin);
    s.update(0);

    s.lo ^ s.hi
}