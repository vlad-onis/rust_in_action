extern crate core;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc::crc32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use std::io::{BufWriter, Read, Seek, SeekFrom};
use std::path::Path;

type ByteString = Vec<u8>;
type ByteStr = [u8];

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyValuePair {
    key: ByteString,
    value: ByteString,
}

#[derive(Debug)]
pub struct ActionKV {
    f: File,
    index: HashMap<ByteString, u64>,
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
            let position = f.seek(SeekFrom::Current(0))?;
            let maybe_kv = ActionKV::process_record(&mut f);
            let kv = match maybe_kv {
                Ok(kv) => kv,
                Err(e) => match e.kind() {
                    io::ErrorKind::UnexpectedEof => {
                        break;
                    }
                    _ => return Err(e),
                },
            };

            self.index.insert(kv.key, position);
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

        let _entry_size = f.by_ref().take(data_len as u64).read_to_end(&mut data)?;

        debug_assert_eq!(data.len() as usize, data_len as usize); // Runtime check for debug builds

        let check_sum = crc32::checksum_ieee(&data);
        if check_sum != saved_check_sum {
            panic!(
                "Data corruption encountered({:08x} != {:08x}",
                check_sum, saved_check_sum
            );
        }

        let value = data.split_off(key_len as usize); // Split a Vec in 2 an n
        let key = data;

        Ok(KeyValuePair { key, value })
    }

    pub fn insert(&mut self, key: &ByteStr, val: &ByteStr) -> io::Result<()> {
        let position = self.insert_ignore_index(key, val)?;
        self.index.insert(key.to_vec(), position);

        Ok(())
    }

    pub fn insert_ignore_index(&mut self, key: &ByteStr, val: &ByteStr) -> io::Result<u64> {
        let mut f = BufWriter::new(&mut self.f);
        let key_len = key.len();
        let val_len = val.len();
        let mut tmp = ByteString::with_capacity(key_len + val_len);

        for byte in key {
            tmp.push(byte.to_owned());
        }

        for byte in val {
            tmp.push(byte.to_owned());
        }

        let checksum = crc32::checksum_ieee(&tmp);

        let next_byte = SeekFrom::End(0);
        let current_position = f.seek(SeekFrom::Current(0))?;
        f.seek(next_byte)?;
        f.write_u32::<LittleEndian>(checksum)?;
        f.write_u32::<LittleEndian>(key_len as u32)?;
        f.write_u32::<LittleEndian>(val_len as u32)?;
        f.write_all(&tmp)?;
        Ok(current_position)
    }
}

#[cfg(test)]
pub mod tests {
    use super::ActionKV;
    use crate::ByteStr;
    use byteorder::{LittleEndian, WriteBytesExt};
    use std::fs::{File, OpenOptions};
    use std::io::{Seek, SeekFrom, Write};
    use std::path::Path;
    use std::{fs, io};

    #[test]
    pub fn open_file() {
        let open_result = ActionKV::open(Path::new("test_data/test"));
        assert!(open_result.is_ok());
    }

    #[test]
    pub fn test_process_record() {
        let path = Path::new("test_data/test_file");
        if path.exists() {
            fs::remove_file(path).expect("Failed to delete file");
        }
        let written = write_hardcoded_bitcask(path, b"vlad", b"onis");
        assert!(written.is_ok());

        let mut f = File::open(path).unwrap();
        let data = ActionKV::process_record(&mut f).unwrap();

        assert_eq!(data.key, b"vlad");
        assert_eq!(data.value, b"onis");
    }

    #[test]
    pub fn test_load() {
        let path = Path::new("test_data/test_file");
        if path.exists() {
            fs::remove_file(path).expect("Failed to delete file");
        }
        let written = write_hardcoded_bitcask(path, b"vlad", b"onis");
        assert!(written.is_ok());
        let written = write_hardcoded_bitcask(path, b"test", b"data");
        assert!(written.is_ok());

        let mut akv = ActionKV::open(path).unwrap();
        akv.f
            .seek(SeekFrom::Start(0))
            .expect("Could not move cursor");
        assert!(akv.load().is_ok());
        let value = akv.index.get("vlad".as_bytes()).unwrap();
        assert_eq!(&0, value);
    }

    #[test]
    pub fn test_insert() {
        let path = Path::new("test_data/test_file");
        if path.exists() {
            fs::remove_file(path).expect("Failed to delete file");
        }

        let key1 = b"vlad";
        let val1 = b"onis";

        let akv = ActionKV::open(Path::new("test_data/test_file"));
        assert!(akv.is_ok());
        let mut akv = akv.unwrap();

        let res = akv.insert_ignore_index(key1, val1);
        assert!(res.is_ok());

        akv.f
            .seek(SeekFrom::Start(0))
            .expect("Could not move cursor");

        let record = ActionKV::process_record(&mut akv.f);
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.key, b"vlad");
        assert_eq!(record.value, b"onis");
    }

    pub fn write_hardcoded_bitcask(path: &Path, key: &ByteStr, val: &ByteStr) -> io::Result<u8> {
        let mut to_write = vec![];

        let mut f = open(path)?;

        let key_len: u32 = key.len() as u32;
        let val_len: u32 = val.len() as u32;

        let data = format!(
            "{}{}",
            String::from_utf8_lossy(key),
            String::from_utf8_lossy(val)
        );
        let check_sum = crc::crc32::checksum_ieee(data.as_bytes());

        to_write.write_u32::<LittleEndian>(check_sum)?;
        to_write.write_u32::<LittleEndian>(key_len)?;
        to_write.write_u32::<LittleEndian>(val_len)?;

        for byte in key {
            to_write.write_u8(*byte)?;
        }

        for byte in val {
            to_write.write_u8(*byte)?;
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
