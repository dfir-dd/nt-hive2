// this is a translation of c code https://github.com/floodyberry/Marvin32/blob/master/Marvin32.c to rust code.
// Marvin32 is a structure containing the current state of the Marvin32
// hash. It has two fields, `lo` and `hi`, which are both u32 values.
#[derive(Debug, Clone, Copy)]
pub struct Marvin32 {
    lo: u32,
    hi: u32,
}

impl Marvin32 {
    pub fn new(seed:u64)-> Self{
        Self {
            lo: (seed & 0xFFFF_FFFF) as u32,
            hi: (seed >> 32) as u32,
        }
    }
    
    // convert_to_le interprets a slice of four bytes as a little-endian u32 value.
    fn convert_to_le(&self,p: &[u8]) -> u32 {
        (p[0] as u32)
            | ((p[1] as u32) << 8)
            | ((p[2] as u32) << 16)
            | ((p[3] as u32) << 24)
    }

    // rotated_left returns the value of `x` rotated left by `k` bits.
    fn rotated_left(&self, x: u32, k: u32) -> u32 {
        (x << k) | (x >> (32 - k))
    }
        // mix updates the Marvin32State with a new value `value`.
    fn mix(&mut self, value: u32) {
        self.lo = self.lo.wrapping_add(value);
        self.hi ^= self.lo;
        self.lo = self.rotated_left(self.lo, 20).wrapping_add( self.hi);
        self.hi = self.rotated_left(self.hi, 9) ^ self.lo;
        self.lo = self.rotated_left(self.lo, 27).wrapping_add(self.hi);
        self.hi = self.rotated_left(self.hi, 19);
    }
    // marvin32_hash computes the Marvin32 hash of the input slice `data`, using the
// given seed value `seed`. It returns the resulting hash as a u32 value.
    pub fn marvin32_hash(&mut self, data: &[u8]) -> u32 {
        // let mut st = 
    
        let len = data.len();
        let mut i = 0;
    
        while i + 4 <= len {
            self.mix(self.convert_to_le(&data[i..i + 4]));
            i += 4;
        }
    
        let mut fin = 0x80u32;
    
        for j in i..len {
            fin = (fin << 8) | (data[j] as u32);
        }
    
        self.mix(fin);
        self.mix(0);
    
        self.lo ^ self.hi
    }
}
