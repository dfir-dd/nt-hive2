use std::{
    cell::RefCell,
    io::{Seek, SeekFrom},
    rc::Rc,
};

use binread::BinReaderExt;

use crate::{hivebin::HiveBin, CleanHive, Hive};

pub(crate) struct HiveBinIterator<B>
where
    B: BinReaderExt,
{
    hive: Rc<RefCell<Hive<B, CleanHive>>>,
    expected_end: u64,
    end_of_file: u64,
}

impl<B> From<Hive<B, CleanHive>> for HiveBinIterator<B>
where
    B: BinReaderExt,
{
    fn from(hive: Hive<B, CleanHive>) -> Self {
        let hive = Rc::new(RefCell::new(hive));
        let end_of_file = hive.borrow_mut().seek(SeekFrom::End(0)).unwrap();
        Self {
            hive,

            // this is where we start reading.
            // we explicitely seek to this position in next()
            expected_end: 0, 
            end_of_file
        }
    }
}

impl<B> Iterator for HiveBinIterator<B>
where
    B: BinReaderExt,
{
    type Item = HiveBin<B>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.expected_end >= self.end_of_file {
            return None;
        }

        /* we could continuously read the HiveBin, because there is no gap
        between them. But, the HiveBin struct only consumes the bytes of the
        HiveBin header. Because we do not know if all the cells in the hive
        have been read, we explicitely seek to the beginning of the next hivebin
        */
        let current_start = self
            .hive
            .borrow_mut()
            .seek(SeekFrom::Start(self.expected_end))
            .unwrap();
        
        match HiveBin::new(Rc::clone(&self.hive)) {
            Ok(hivebin) => {
                self.expected_end = current_start + *hivebin.size() as u64;
                Some(hivebin)
            }
            Err(why) => {
                log::error!("{why}");
                None
            }
        }
    }
}
