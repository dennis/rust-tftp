use std::fs::File;
use std::io::Read;
use std::io::{Seek, SeekFrom};
use std::iter::Iterator;

pub trait TftpReadStream {
    fn get_block(&mut self, start : usize, length: usize) -> Result<Vec<u8>, ()>;
}

pub struct NullStream;
impl TftpReadStream for NullStream {
    fn get_block(&mut self, _ : usize, _: usize) -> Result<Vec<u8>, ()> {
        let empty : Vec<u8> =  Vec::new();
        Ok(empty)
    }
}

pub struct StringStream {
    bytes: Vec<u8>
}

impl StringStream {
    pub fn new(input : String) -> StringStream {
        StringStream {
            bytes: input.as_bytes().iter().cloned().collect()
        }
    }
}

impl TftpReadStream for StringStream {
    fn get_block(&mut self, start : usize, length: usize) -> Result<Vec<u8>, ()> {
        let mut end = start+length;
        if start > self.bytes.len() || start > end {
            Err(())
        }
        else {
            if self.bytes.len() < end {
                end = self.bytes.len();
            }
            println!("    ByteStream: start={}, end={}, bytes-length={}", start, end, self.bytes.len());
            Ok(self.bytes[start..end].iter().cloned().collect())
        }
    }
}

pub struct FileStream {
    file: File
}

impl FileStream {
    pub fn new(input : File) -> FileStream {
        FileStream {
            file: input
        }
    }
}

impl TftpReadStream for FileStream {
    fn get_block(&mut self, start: usize, length: usize) -> Result<Vec<u8>, ()> {
        if let Ok(_) = self.file.seek(SeekFrom::Start(start as u64)) {
            let mut result : Vec<u8> = Vec::new();

            for b in self.file.by_ref().take(length as u64).bytes() {
                if let Ok(b) = b {
                    result.push(b);
                }
                else {
                    break;
                }
            }
            Ok(result)
        }
        else {
            Err(())
        }
    }
}
