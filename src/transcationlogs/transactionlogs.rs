use super:: transactionlogsentry::TransctionLogs;
use crate::hive::*;
use anyhow::Ok;
use binread::BinResult;
use binread::{io::Cursor, BinReaderExt};
use std::io::{ SeekFrom};


use std::{fs::File, path::Path};



#[derive(Debug, Clone)]
pub struct RecoverHive {
    new_hive: Vec<u8>,
    trnslogs: Vec<TransctionLogs>,
    new_sequence_number: u32,
    primary_block: HiveBaseBlock,
}

impl RecoverHive {
    fn compute_checksum(&self, buffer: &[u8]) -> u32 {
        let mut chk: u32 = 0;
        // compute over first 508 bytes of block
        (0..127).for_each(|i| {
            let pos = i * 4;
            let mut buf = Cursor::new(&buffer[pos..pos + 4]);
            let chunk: u32 = buf.read_le().unwrap();
            chk ^= chunk;
        });
        chk
    }

    pub fn is_dirty(&self, mut calc_csum: u32) -> bool {
        let p_s_n = self.primary_block.primary_sequence_number;
        let s_s_n = self.primary_block.secondary_sequence_number;
        let check_sum = self.primary_block.checksum;
        calc_csum = check_sum + 1;
        if p_s_n != s_s_n {
            true
        } else if check_sum != calc_csum {
            true
        } else {
            false
        }
    }

    fn check_exists(&self, file: &Path) -> bool {
        if file.exists() {
            true
        } else {
            false
        }
    }
    pub fn new() -> Self {
        let trnslogs: Vec<TransctionLogs> = Vec::new();
        let recoved_hive_file: Vec<u8> = Vec::new();
        let primary_block: HiveBaseBlock = HiveBaseBlock::default();
        

        RecoverHive {
            new_hive: recoved_hive_file,
            trnslogs,
            new_sequence_number: 0,
            primary_block,
        }
    }

    pub fn recover_hive<B: BinReaderExt>(
        &mut self,
        mut data: Hive<B>,
        path_logs: &str,
    ) ->&Vec<u8>{
        //read the primary header and get the squence number and the checusum

        self.primary_block = data.base_block.unwrap();

        let hive_sec_seq: u32 = self.primary_block.secondary_sequence_number;

        //read the first 4096 bytes and calc the check sum for intgrity and recover
        let mut buffer: [u8; 4096] = [0; 4096];
        data.data.seek(SeekFrom::Start(0)).unwrap();
        data.data.read_exact(&mut buffer).unwrap();
        data.data.seek(SeekFrom::Start(0)).unwrap();

        let ck_sm = self.compute_checksum(&buffer);
        let hivechecksum = if self.primary_block.checksum == ck_sm {
            true
        } else {
            false
        };
        let mut data_primary = data.data;

        let dirty = self.is_dirty(ck_sm);
        if dirty {
            self.process_recovery(path_logs, hivechecksum, hive_sec_seq);
            self.replay_dirtylogs(data_primary);
        }
        
        &self.new_hive
       
    }

    // too many unwrap on this function that need to be fixed later also this function can be optimized and decalred very well 
    fn process_recovery(&mut self, path_logs: &str, hivechecksum: bool, hive_sec_seq: u32) {
        let a = format!("{}{}", path_logs, ".LOG1");
        let b = format!("{}{}", path_logs, ".LOG2");

        let log_one = Path::new(&a);
        let log_two = Path::new(&b);
        let log_ends_one: bool = self.check_exists(&log_one);
        let log_ends_two: bool = self.check_exists(&log_two);
        let mut header_log_one_block: &HiveBaseBlock = &HiveBaseBlock::default();
        let mut header_log_two_block: &HiveBaseBlock = &HiveBaseBlock::default();

        let mut first_log: Hive<File>;
        let mut second_log: Hive<File>;
        let mut singlelog: Hive<File>;

        if log_ends_one && log_ends_two {
            let header_log_one = self.read_logs(&log_one);
            header_log_one_block = header_log_one.base_block.as_ref().unwrap();

            let header_log_two = self.read_logs(&log_two);
            header_log_two_block = header_log_two.base_block.as_ref().unwrap();

            if (header_log_one_block.primary_sequence_number
                >= header_log_two_block.primary_sequence_number)
            {
                first_log = header_log_two;
                second_log = header_log_one;
            } else {
                first_log = header_log_one;
                second_log = header_log_two;
            }

            if (hivechecksum
                && first_log
                    .base_block
                    .as_ref()
                    .unwrap()
                    .primary_sequence_number
                    >= hive_sec_seq)
            {
                let (log_data, newsequencenumber) = self.read_log_data(
                    &first_log.data,
                    first_log.base_block.unwrap().primary_sequence_number,
                );
                for d in log_data {
                    self.trnslogs.push(d);
                }
                self.new_sequence_number = newsequencenumber;
            } else {
                let (log_data, newsequencenumber) = self.read_log_data(
                    &second_log.data,
                    second_log
                        .base_block
                        .as_ref()
                        .unwrap()
                        .primary_sequence_number,
                );
                for d in log_data {
                    self.trnslogs.push(d);
                }
                self.new_sequence_number = newsequencenumber;
            }

            if (second_log
                .base_block
                .as_ref()
                .unwrap()
                .primary_sequence_number
                == self.new_sequence_number + 1
                && second_log
                    .base_block
                    .as_ref()
                    .unwrap()
                    .primary_sequence_number
                    > hive_sec_seq)
            {
                let (log_data, newsequencenumber) = self.read_log_data(
                    &second_log.data,
                    second_log
                        .base_block
                        .as_ref()
                        .unwrap()
                        .primary_sequence_number,
                );

                for d in log_data {
                    self.trnslogs.push(d);
                }
                self.new_sequence_number = newsequencenumber;
            }
        } else if log_ends_one {
            let log_one = self.read_logs(&log_one);
            if (hivechecksum
                && log_one.base_block.as_ref().unwrap().primary_sequence_number >= hive_sec_seq)
            {
                let (log_data, newsequencenumber) = self.read_log_data(
                    &log_one.data,
                    log_one.base_block.unwrap().primary_sequence_number,
                );
                for d in log_data {
                    self.trnslogs.push(d);
                }
                self.new_sequence_number = newsequencenumber;
            }
        } else if log_ends_two {
            let header_two = self.read_logs(&log_two);
            if (hivechecksum
                && header_two
                    .base_block
                    .as_ref()
                    .unwrap()
                    .primary_sequence_number
                    >= hive_sec_seq)
            {
                let (log_data, newsequencenumber) = self.read_log_data(
                    &header_two.data,
                    header_two.base_block.unwrap().primary_sequence_number,
                );
                for d in log_data {
                    self.trnslogs.push(d);
                }
                self.new_sequence_number = newsequencenumber;
            }
        }
    }

    /// Read the log to the end and update the primary hive
    fn read_log_data(&self, mut file: &File, prim_sq_num: u32) -> (Vec<TransctionLogs>, u32) {
        let fixedhive = TransctionLogs::new(&mut file, prim_sq_num).unwrap();
        fixedhive
    }
    ///Recover the logs using the log vec and the primary hive
    fn replay_dirtylogs<T: BinReaderExt>(&mut self, mut data: T) {
        let mut hive_primary_file: Vec<u8> = Vec::new();
        data.read_to_end(&mut hive_primary_file).unwrap(); 
        for x in self.trnslogs.iter() {
            for f in x.d_pages.iter() {
                let range =
                    f.primary_offset as usize..f.primary_offset as usize + f.page_size as usize;
               
                hive_primary_file.splice(range, f.data.iter().cloned());
            }
        }
        let  new_sq = self.new_sequence_number.to_le_bytes();
        //update the sequence number 
         hive_primary_file.splice(4..(4+new_sq.len()), new_sq);
         hive_primary_file.splice(8..(8+new_sq.len()), new_sq);

         // I still miss this checksum update but I will do it next time


         self.new_hive = hive_primary_file
         
    } 
    fn read_logs(&self, log: &Path) -> Hive<File> {
        let log_file_bin = File::open(log).unwrap();
        let hive_log = Hive::new(log_file_bin, HiveParseMode::NormalWithBaseBlock).unwrap();
        hive_log
    }
}
