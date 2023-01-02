use crate::nk::{KeyNodeFlags, KeyNodeWithMagic};
use crate::transcationlogs::TransactionLog;
use crate::{nk::KeyNode, CellIterator};
use crate::{Cell, CellFilter, CellLookAhead, HiveParseMode, Offset};
use binread::{BinRead, BinReaderExt, BinResult};
use memoverlay::MemOverlay;
use std::io::{Cursor, ErrorKind, Read, Seek, SeekFrom};

use super::base_block::HiveBaseBlock;
use super::base_block_raw::HiveBaseBlockRaw;

/// Represents a registry hive file.
///
/// Because most offsets in a registry hive file are relative to the start of the hive bins data,
/// this struct provides a own [Seek] and [Read] implementation, which can work directly
/// with those kinds of offsets. You don't know where the hive bins data starts, because [Hive] knows
/// it (this information is stored in the hive base block). To parse data from within the hive bins data,
/// use [Hive] as reader and use offsets read from the hive data structures.
///
/// The structure of hive files is documented at <https://github.com/msuhanov/regf/blob/master/Windows%20registry%20file%20format%20specification.md#format-of-primary-files>
#[derive(Debug)]
pub struct Hive<B>
where
    B: BinReaderExt,
{
    pub data: MemOverlay<B>,
    pub(crate) base_block: Option<HiveBaseBlock>,
    data_offset: u32,
    root_cell_offset: Option<Offset>,
}

impl<B> Hive<B>
where
    B: BinReaderExt,
{
    /// creates a new [Hive] object. This includes parsing the HiveBaseBlock and determining
    /// the start of the hive bins data.
    pub fn new(mut data: B, parse_mode: HiveParseMode) -> BinResult<Self> {
        data.seek(SeekFrom::Start(0))?;
        let mut data = MemOverlay::from(data);
        let me = match parse_mode {
            HiveParseMode::Raw => Self {
                data,
                base_block: None,
                data_offset: 0x1000,
                root_cell_offset: None,
            },
            HiveParseMode::Normal(offset) => Self {
                data,
                base_block: None,
                data_offset: 0x1000,
                root_cell_offset: Some(offset),
            },
            HiveParseMode::NormalWithBaseBlock => {
                /* preread the baseblock data to prevent seeking */
                let mut baseblock_data = [0; 4096];
                data.read_exact(&mut baseblock_data)?;

                Self::validate_checksum(&baseblock_data)?;

                /* read baseblock */
                let mut baseblock_cursor = Cursor::new(baseblock_data);
                let base_block: HiveBaseBlock = baseblock_cursor.read_le().unwrap();
                let data_offset = data.stream_position()? as u32;
                let root_cell_offset = *base_block.root_cell_offset();
                Self {
                    data,
                    base_block: Some(base_block),
                    data_offset,
                    root_cell_offset: Some(root_cell_offset),
                }
            }
        };

        Ok(me)
    }

    fn validate_checksum(baseblock_data: &[u8; 4096]) -> BinResult<()> {
        let mut cursor = Cursor::new(baseblock_data);
        let _: HiveBaseBlockRaw = match cursor.read_le() {
            Ok(r) => r,
            Err(why) => {
                log::error!("invalid checksum detected");
                return Err(why);
            }
        };
        Ok(())
    }

    pub fn is_primary_file(&self) -> bool {
        if let Some(base_block) = &self.base_block {
            *base_block.file_type() == 0
        } else {
            false
        }
    }

    /// Is this really needed???
    pub fn enum_subkeys(
        &mut self,
        callback: fn(&mut Self, &KeyNode) -> BinResult<()>,
    ) -> BinResult<()> {
        let root_key_node = self.root_key_node()?;
        callback(self, &root_key_node)?;
        Ok(())
    }

    /// returns the root key of this registry hive file
    pub fn root_key_node(&mut self) -> BinResult<KeyNode> {
        let mkn: KeyNodeWithMagic = self.read_structure(self.root_cell_offset())?;
        Ok(mkn.into())
    }

    /// reads a data structure from the given offset. Read the documentation of [Cell]
    /// for a detailled discussion
    ///
    /// # Usage
    ///
    /// ```
    /// # use std::error::Error;
    /// # use std::fs::File;
    /// use nt_hive2::*;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let hive_file = File::open("tests/data/testhive")?;
    /// # let mut hive = Hive::new(hive_file, HiveParseMode::NormalWithBaseBlock)?;
    /// # let offset = hive.root_cell_offset();
    /// let my_node: KeyNodeWithMagic = hive.read_structure(offset)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_structure<T>(&mut self, offset: Offset) -> BinResult<T>
    where
        T: BinRead<Args = ()> + std::convert::From<crate::Cell<T, ()>>,
    {
        log::trace!(
            "reading cell of type {} from offset {:08x} (was: {:08x})",
            std::any::type_name::<T>(),
            offset.0 + self.data_offset,
            offset.0
        );

        self.seek(SeekFrom::Start(offset.0.into()))?;
        let cell: Cell<T, ()> = self.read_le().unwrap();
        assert!(cell.is_allocated());
        Ok(cell.into())
    }

    /// returns the start of the hive bins data
    pub fn data_offset(&self) -> &u32 {
        &self.data_offset
    }

    /// returns the offset of the root cell
    pub fn root_cell_offset(&self) -> Offset {
        match &self.base_block {
            None => self.root_cell_offset.unwrap(),
            Some(base_block) => *base_block.root_cell_offset(),
        }
    }

    pub fn find_root_celloffset(self) -> Option<Offset> {
        let iterator = self
            .into_cell_iterator(|_| ())
            .with_filter(CellFilter::AllocatedOnly);
        for cell in iterator {
            if let CellLookAhead::NK(nk) = cell.content() {
                if nk.flags.contains(KeyNodeFlags::KEY_HIVE_ENTRY) {
                    return Some(*cell.offset());
                }
            }
        }
        None
    }

    pub fn into_cell_iterator<C>(self, callback: C) -> CellIterator<B, C>
    where
        C: Fn(u64),
    {
        CellIterator::new(self, callback)
    }

    pub fn data_size(&self) -> u32 {
        match &self.base_block {
            None => todo!(),
            Some(base_block) => *base_block.data_size(),
        }
    }

    pub fn with_transaction_log(mut self, log: TransactionLog) -> std::io::Result<Self> {
        let mut sequence_number = match self.base_block {
            Some(ref base_block) => *base_block.primary_sequence_number(),
            None => 0,
        };

        for entry in log.log_entries() {
            if sequence_number != 0 && *entry.sequence_number() != sequence_number + 1 {
                log::warn!("abort applying transaction logs at sequence number {sequence_number}");
                log::warn!(
                    "next log entry has transaction number: {}",
                    entry.sequence_number()
                );
                break;
            }
            sequence_number = *entry.sequence_number();

            for (reference, page) in entry
                .dirty_pages_references()
                .iter()
                .zip(entry.dirty_pages())
            {
                self.data.add_bytes_at(
                    (log.base_block().root_cell_offset().0 + reference.offset().0).into(),
                    page,
                )?;
            }
        }
        Ok(self)
    }
}

impl<B> Read for Hive<B>
where
    B: BinReaderExt,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.data.read(buf)
    }
}

impl<B> Seek for Hive<B>
where
    B: BinReaderExt,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_offset = match pos {
            SeekFrom::Start(dst) => self
                .data
                .seek(SeekFrom::Start(dst + self.data_offset as u64))?,
            SeekFrom::End(_) => self.data.seek(pos)?,
            SeekFrom::Current(_) => self.data.seek(pos)?,
        };
        if new_offset < self.data_offset as u64 {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("tried seek to invalid offset: {:?}", pos),
            ));
        }
        Ok(new_offset - self.data_offset as u64)
    }
}
