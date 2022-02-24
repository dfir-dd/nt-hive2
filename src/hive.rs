use std::io::{Read, Seek};
pub struct Hive<RS> where RS: Read+Seek {
    data: RS,
}

impl<RS> Hive<RS> where RS: Read+Seek{
    pub fn open(data: RS) -> Self {
        Self {
            data
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::*;
    use std::io;

    #[test]
    fn enum_subkeys() {
        let testhive = crate::helpers::tests::testhive_vec();
        let hive = Hive::open(io::Cursor::new(testhive));
        //assert!(hive.enum_subkeys(|k| Ok(())).is_ok());
    }
}