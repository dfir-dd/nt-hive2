use binread::BinRead;
use binwrite::BinWrite;
use byteorder::{WriteBytesExt, LittleEndian};
use num_traits::ToPrimitive;

/// <https://github.com/libyal/libregf/blob/main/documentation/Windows%20NT%20Registry%20File%20(REGF)%20format.asciidoc>
#[derive(BinRead, PartialEq, Eq, Debug, Clone, Copy, num_derive::ToPrimitive)]
#[br(repr=u32)]
pub enum FileType {
    /// Registry hive file
    HiveFile = 0,

    /// Transaction log variant 1, seen on Windows XP (SP2, SP3), Vista, Windows 7 and 8.0
    TransactionLogVariant1 = 1,

    /// Transaction log variant 2, seen on Windows NT 3.51, NT 4.0 and 2000
    TransactionLogVariant2 = 2,

    /// Transaction log variant 6, seen on Windows 8.1, Server 2012 R2 and Windows 10
    TransactionLogVariant3 = 6
}

impl Default for FileType {
    fn default() -> Self {
        Self::HiveFile
    }
}

impl BinWrite for FileType {
    fn write_options<W: std::io::Write>(&self, writer: &mut W, _options: &binwrite::WriterOption) -> std::io::Result<()> {
        writer.write_u32::<LittleEndian>(self.to_u32().unwrap())?;
        Ok(())
    }
}