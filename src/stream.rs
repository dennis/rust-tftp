use std::fs::File;
use std::io::Read;

pub trait TftpReadStream {
    fn get_block(&mut self, start : usize, length: usize) -> Result<Vec<u8>, ()>;
}

pub trait TftpReadStreamProvider {
    fn get_tftp_read_stream(&mut self) -> Result<Box<TftpReadStream>, ()>;
}

pub struct NullStream;
impl TftpReadStream for NullStream {
    fn get_block(&mut self, start : usize, length: usize) -> Result<Vec<u8>, ()> {
        let empty : Vec<u8> =  Vec::new();
        Ok(empty)
    }
}

impl TftpReadStreamProvider for NullStream {
    fn get_tftp_read_stream(&mut self) -> Result<Box<TftpReadStream>, ()> {
        Ok(Box::new(NullStream))
    }
}

struct StringStream {
    bytes : Vec<u8>,
}

impl TftpReadStream for StringStream {
    fn get_block(&mut self, start: usize, length: usize) -> Result<Vec<u8>, ()> {
        let mut end = start+length;
        if start > self.bytes.len() || start > end {
            Err(())
        }
        else {
            if self.bytes.len() < end {
                end = self.bytes.len();
            }
            println!("    StringStream: start={}, end={}, bytes-length={}", start, end, self.bytes.len());
            Ok(self.bytes[start..end].iter().cloned().collect())
        }
    }
}
impl TftpReadStreamProvider for String {
    fn get_tftp_read_stream(&mut self) -> Result<Box<TftpReadStream>, ()> {
        Ok(Box::new(StringStream {
            bytes: self.as_bytes().iter().cloned().collect()
        }))
    }
}

impl TftpReadStreamProvider for File {
    fn get_tftp_read_stream(&mut self) -> Result<Box<TftpReadStream>, ()> {
        // FIXME - we load the whole file into memory
        let mut s = String::new();
        self.read_to_string(&mut s).unwrap();

        Ok(Box::new(StringStream {
            bytes: s.as_bytes().iter().cloned().collect()
        }))
    }
}
