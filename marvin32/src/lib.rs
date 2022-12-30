mod marvin32;

pub use crate::marvin32::Marvin32;

#[cfg(test)]
mod tests {
    use super::Marvin32;
    use std::hash::Hasher;

    fn run_marvin32(expected: u64, bytes: Vec<&[u8]>, seed: u64) {
        let mut hasher = Marvin32::new(seed);

        for b in bytes {
            let bytes: Vec<_> = b.as_ref().into();
            hasher.write(&bytes[..]);
        }

        let hash = hasher.finish();
        assert_eq!(expected, hash, "0x{expected:08x} != 0x{hash:08x}");
    }

    #[test]
    fn test_marvin32_1() {
        run_marvin32(
            0xba627c81,
            vec![&[
                b'A', 0, b'b', 0, b'c', 0, b'd', 0, b'e', 0, b'f', 0, b'g', 0,
            ]],
            0x5D70D359C498B3F8,
        );
    }

    #[test]
    fn test_marvin32_2() {
        run_marvin32(
            0xba627c81,
            vec![
                &(vec![b'A', 0, b'b', 0, b'c'][..]),
                &(vec![0, b'd', 0, b'e', 0, b'f', 0, b'g', 0][..]),
            ],
            0x5D70D359C498B3F8,
        );
    }

    #[test]
    fn test_marvin32_3() {
        run_marvin32(0xb00892ac, vec![&[]], 0x82EF4D887A4E55C5);
    }

    #[test]
    fn test_marvin32_4() {
        run_marvin32(0xf41a608e, vec!["h".as_bytes()], 0x82EF4D887A4E55C5);
    }

    #[test]
    fn test_marvin32_5() {
        run_marvin32(0x11107c6b, vec!["he".as_bytes()], 0x82EF4D887A4E55C5);
    }

    #[test]
    fn test_marvin32_6() {
        run_marvin32(0x24056a46, vec!["hel".as_bytes()], 0x82EF4D887A4E55C5);
    }

    #[test]
    fn test_marvin32_7() {
        run_marvin32(0x7f91e021, vec!["hell".as_bytes()], 0x82EF4D887A4E55C5);
    }

    #[test]
    fn test_marvin32_8() {
        run_marvin32(
            0x00c18515,
            vec!["hello, world!".as_bytes()],
            0x82EF4D887A4E55C5,
        );
        run_marvin32(
            0x00c18515,
            vec!["hello, ".as_bytes(), "world!".as_bytes()],
            0x82EF4D887A4E55C5,
        );
        run_marvin32(
            0x00c18515,
            vec!["hello, w".as_bytes(), "orld!".as_bytes()],
            0x82EF4D887A4E55C5,
        );
        run_marvin32(
            0x00c18515,
            vec!["hello, wo".as_bytes(), "rld!".as_bytes()],
            0x82EF4D887A4E55C5,
        );
    }
}
