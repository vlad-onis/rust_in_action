extern crate core;

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::Path;
use serde::{ Serialize, Deserialize };
use std::io;
use std::io::{Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use crc::crc32;

type ByteString = Vec<u8>;

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyValuePair {
    key: ByteString,
    value: ByteString
}

#[derive(Debug)]
pub struct ActionKV {
    f: File,
    index: HashMap<ByteString, ByteString>,
}

impl ActionKV {

    pub fn open(path: &Path) -> io::Result<ActionKV> {
        let f = OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .create(true)
            .open(path)?;

        let index = HashMap::new();
        Ok(ActionKV { f, index })
    }

    pub fn load(&mut self) -> io::Result<()> {

        let mut f = io::BufReader::new(&mut self.f);

        loop {
            let _position = f.seek(SeekFrom::Current(0))?;
            let maybe_kv = ActionKV::process_record(&mut f);
            let kv = match maybe_kv {
                Ok(kv) => kv,
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::UnexpectedEof => {
                            break;
                        }
                        _ => return Err(e),
                    }
                }
            };

            self.index.insert(kv.key, kv.value);
        }

        Ok(())
    }

    /// Format of a record is: checksum(u32), key_len(u32), val_len(u32), key([u8, key_len]),
    /// value([u8, val_len])
    fn process_record<R: Read>(f: &mut R) -> io::Result<KeyValuePair> {

        let saved_check_sum = f.read_u32::<LittleEndian>()?;
        let key_len = f.read_u32::<LittleEndian>()?;
        let val_len = f.read_u32::<LittleEndian>()?;
        let data_len = key_len + val_len;

        let mut data = ByteString::with_capacity(data_len as usize);

        let _entry_size = f.by_ref()
            .take(data_len as u64)
            .read_to_end(&mut data)?;

        debug_assert_eq!(data.len() as usize, data_len as usize); // Runtime check for debug builds

        let check_sum = crc32::checksum_ieee(&data);
        if check_sum != saved_check_sum {
            panic!("Data corruption encountered({:08x} != {:08x}", check_sum, saved_check_sum);
        }

        let value = data.split_off(key_len as usize); // Split a Vec in 2 an n
        let key = data;

        Ok(KeyValuePair {
            key,
            value
        })
    }
}

#[cfg(test)]
pub mod tests {
    use std::io;
    use std::io::Write;
    use std::path::Path;
    use byteorder::{ LittleEndian, WriteBytesExt };
    use std::fs::{File, OpenOptions};
    use super::ActionKV;

    #[test]
    pub fn open_file() {
        let open_result = ActionKV::open(Path::new("test_data/test"));
        assert!(open_result.is_ok());
    }

    #[test]
    pub fn test_process_record() {
        let path = Path::new("test_data/test_file");
        let written = write_hardcoded_bitcask(path);
        assert!(written.is_ok());

        let mut f = File::open(path).unwrap();
        let data = ActionKV::process_record(&mut f).unwrap();

        assert_eq!(data.key, b"vlad");
        assert_eq!(data.value, b"onis");
    }

    #[test]
    pub fn test_load() {
        let path = Path::new("test_data/test_file");
        let written = write_hardcoded_bitcask(path);
        assert!(written.is_ok());

        let mut akv = ActionKV::open(path).unwrap();
        assert!(akv.load().is_ok());
        let value = String::from_utf8_lossy(akv.index.get("vlad".as_bytes()).unwrap());
        assert_eq!(String::from("onis"), value);
    }

    pub fn write_hardcoded_bitcask(path: &Path) -> io::Result<u8> {

        let mut to_write = vec![];

        let mut f = open(path)?;

        let key_len: u32 = 4;
        let val_len: u32 = 4;
        let key: [u8; 4] = b"vlad".to_owned();
        let val: [u8; 4] = b"onis".to_owned();

        let data = b"vladonis";
        let check_sum = crc::crc32::checksum_ieee(data);

        to_write.write_u32::<LittleEndian>(check_sum)?;
        to_write.write_u32::<LittleEndian>(key_len)?;
        to_write.write_u32::<LittleEndian>(val_len)?;

        for byte in key {
            to_write.write_u8(byte)?;
        }

        for byte in val {
            to_write.write_u8(byte)?;
        }

        f.write(to_write.as_ref())?;

        Ok(0)
    }

    fn open(path: &Path) -> io::Result<File> {
        let f = OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .create(true)
            .open(path)?;

        Ok(f)
    }
}