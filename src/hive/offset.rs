use binread::BinRead;
use binwrite::BinWrite;
use byteorder::{WriteBytesExt, LittleEndian};

/// represents an offset (usually a 32bit value) used in registry hive files
#[derive(BinRead, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Offset(pub u32);

impl BinWrite for Offset {
    fn write_options<W: std::io::Write>(&self, writer: &mut W, _options: &binwrite::WriterOption) -> std::io::Result<()> {
        writer.write_u32::<LittleEndian>(self.0)
    }
}