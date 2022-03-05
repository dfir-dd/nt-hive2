use std::io;
use displaydoc::Display;

pub type Result<T, E = NtHiveError> = core::result::Result<T, E>;


#[derive(Clone, Debug, Display)]
pub enum NtHiveError {
    /// The checksum in the base block should be {expected}, but it is {actual}
    InvalidChecksum { expected: u32, actual: u32 },
    /// The data at offset {offset:#010x} should have a size of {expected} bytes, but it only has {actual} bytes
    InvalidDataSize {
        offset: usize,
        expected: usize,
        actual: usize,
    },
    /// The 4-byte signature field at offset {offset:#010x} should contain {expected:?}, but it contains {actual:?}
    InvalidFourByteSignature {
        offset: usize,
        expected: &'static [u8],
        actual: [u8; 4],
    },
    /// The struct at offset {offset:#010x} should have a size of {expected} bytes, but only {actual} bytes are left in the slice
    InvalidHeaderSize {
        offset: usize,
        expected: usize,
        actual: usize,
    },
    /*
    /// Expected one of the key value data types {expected:?}, but found {actual:?}
    InvalidKeyValueDataType {
        expected: &'static [KeyValueDataType],
        actual: KeyValueDataType,
    },
     */
    /// The size field at offset {offset:#010x} specifies {expected} bytes, but only {actual} bytes are left in the slice
    InvalidSizeField {
        offset: usize,
        expected: usize,
        actual: usize,
    },
    /// The size field at offset {offset:#010x} specifies {size} bytes, but they are not aligned to the expected {expected_alignment} bytes
    InvalidSizeFieldAlignment {
        offset: usize,
        size: usize,
        expected_alignment: usize,
    },
    /// The 2-byte signature field at offset {offset:#010x} should contain {expected:?}, but it contains {actual:?}
    InvalidTwoByteSignature {
        offset: usize,
        expected: &'static [u8],
        actual: [u8; 2],
    },
    /// The sequence numbers in the base block do not match ({primary} != {secondary})
    SequenceNumberMismatch { primary: u32, secondary: u32 },
    /// The cell at offset {offset:#010x} with a size of {size} bytes is unallocated
    UnallocatedCell { offset: usize, size: i32 },
    /// The clustering factor in the base block is expected to be {expected}, but it is {actual}
    UnsupportedClusteringFactor { expected: u32, actual: u32 },
    /// The file format in the base block is expected to be {expected}, but it is {actual}
    UnsupportedFileFormat { expected: u32, actual: u32 },
    /// The file type in the base block is expected to be {expected}, but it is {actual}
    UnsupportedFileType { expected: u32, actual: u32 },
    /// The key value data type at offset {offset:#010x} is {actual:#010x}, which is not supported
    UnsupportedKeyValueDataType { offset: usize, actual: u32 },
    /// The version in the base block ({major}.{minor}) is unsupported
    UnsupportedVersion { major: u32, minor: u32 },
    /// any error that may occur during parsing. Because binread::error is not Clone, we use an error message instead
    BinreadError {msg: String },
    /// any IO error that may occur during parsing. Because io::error is not Clone, we use an error message instead
    IOError {msg: String },
    /// indicates that an unexpected IndexRoot has been read
    UnexpectedIndexRoot,
    /// Unable to decode String
    StringEncodingError,
}


#[cfg(feature = "std")]
impl std::error::Error for NtHiveError {}

impl From<binread::Error> for NtHiveError {
    fn from(error: binread::Error) -> Self {
        match error {
            binread::Error::Custom { pos: _, err } =>
                match err.downcast_ref::<NtHiveError>() {
                    None => panic!("unsupported error type"),
                    Some(e) => e.clone()
                }
            binread::Error::BadMagic { pos, found: _ } => Self::BinreadError{msg: format!("invalid magic at {}", pos)},
            binread::Error::AssertFail { pos, message } => Self::BinreadError{msg: format!("assertion failed at at {}: {}", pos, message)},
            binread::Error::Io(why) => Self::BinreadError{msg: format!("IO error: {:?}", why)},
            binread::Error::NoVariantMatch { pos }  => Self::BinreadError{msg: format!("no variant match at {}", pos)},
            binread::Error::EnumErrors { pos, variant_errors: _ }  => Self::BinreadError{msg: format!("enum errors at {}", pos)},
            err => Self::BinreadError{msg: format!("binread error: {:?}", err)}
        }
    }
}

impl From<io::Error> for NtHiveError {
    fn from(error: io::Error) -> Self {
        Self::BinreadError{msg: format!("IO error: {:?}", error)}
    }
}