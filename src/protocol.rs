use std::io;
use std::str;
use std::io::Write;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use packet::{Packet, ErrorCode};

pub struct Protocol;

impl Protocol {
    pub fn encode(packet : Packet) -> Result<Vec<u8>, &'static str> {
        let mut buf : Vec<u8> = Vec::new();

        match packet {
            Packet::ERROR(error_code, error_string) => {
                if let Err(_) = buf.write_u16::<BigEndian>(5) {
                    return Err("Error writing opcode")
                }
                if let Err(_) = buf.write_u16::<BigEndian>(error_code as u16) {
                    return Err("Error writing error_code")
                }
                if let Err(_) = Self::encode_string(&mut buf, error_string) {
                    return Err("Error writing error_message")
                }

                Ok(buf)
            },
            Packet::RRQ(_, _) => {
                unimplemented!();
            },
            Packet::WRQ(_, _) => {
                unimplemented!();
            },
            Packet::ACK(_) => {
                unimplemented!();
            },
            Packet::Data(block_no, data) => {
                if let Err(_) = buf.write_u16::<BigEndian>(3) {
                    return Err("Error writing opcode")
                }
                if let Err(_) = buf.write_u16::<BigEndian>(block_no) {
                    return Err("Error writing block #")
                }
                if let Err(_) = buf.write_all(&data[..]) {
                    return Err("Error writing data")
                }

                Ok(buf)
            },
        }
    }

    fn encode_string(buf : &mut Vec<u8>, string : String) -> Result<&'static str, &'static str>{
        for c in string.chars() {
            if let Err(_) = buf.write_u8(c as u8) {
                return Err("Error writing packet")
            }
        }
        if let Err(_) = buf.write_u8(0u8) {
            return Err("Error writing packet")
        }
        else {
            return Ok("Done")
        }
    }

    pub fn decode(p : &[u8]) -> Result<Packet, String> {
        let mut reader = io::Cursor::new(p);
        let opcode_result = reader.read_u16::<::byteorder::BigEndian>();

        match opcode_result {
            Ok(opcode) => {
                // 1 - RRQ
                // 2 - WRQ
                // 3 - DATA
                // 4 - ACK
                // 5 - ERROR
                match opcode {
                    // RRQ opcode 1, WRQ 2
                    1|2 => {
                        if let Ok(filename) = Self::decode_string(&mut reader) {
                            if let Ok(mode_name) = Self::decode_string(&mut reader) {
                                if opcode == 1 {
                                    return Ok(Packet::RRQ(filename, mode_name))
                                }
                                else { //if opcode == 2
                                    return Ok(Packet::WRQ(filename, mode_name))
                                }
                            }
                        }
                    },
                    // ACK
                    4 => {
                        if let Ok(block_no) = reader.read_u16::<::byteorder::BigEndian>() {
                            return Ok(Packet::ACK(block_no))
                        }
                    }
                    5 => {
                        if let Ok(error_code) = reader.read_u16::<::byteorder::BigEndian>() {
                            if let Ok(error_message) = Self::decode_string(&mut reader) {
                                return Ok(Packet::ERROR(ErrorCode::from_u16(error_code), error_message))
                            }
                        }
                    },
                    opcode => {
                        return Err(format!("Unknown opcode: {}", opcode))
                    }
                }
            },
            Err(_) => {
                return Err("Error decoding opcode".to_string())
            }
        }

        Err("Error parsing packet".to_string())
    }

    fn decode_string<T : ::byteorder::ReadBytesExt>(reader : &mut T) -> Result<String, &str> {
        let mut string_bytes = Vec::new();

        while let Ok(c) = reader.read_u8() {
            if c == 0u8 {
                break;
            }

            string_bytes.push(c);
        }

        match str::from_utf8(&string_bytes[..]) {
            Ok(str) => Ok(str.to_string()),
            Err(_) => Err("Cannot decode string as UTF-8"),
        }
    }
}
