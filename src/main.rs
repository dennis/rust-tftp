extern crate byteorder;

use std::io;
use std::net::UdpSocket;
use std::str;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Write;

const MAX_PACKET_SIZE : usize = 512;

// https://www.ietf.org/rfc/rfc1350.txt

enum ErrorCode {
    NotDefined                 = 0,
    FileNotFound               = 1,
    AccessViolation            = 2,
    DiskFullOrAllocationFailed = 3,
    IllegalTftpOperation       = 4,
    UnknownTransferId          = 5,
    FileAlreadyExists          = 6,
    NoSuchUser                 = 7
}

impl ErrorCode {
    fn from_u16(val : u16) -> ErrorCode {
        match val {
            0 => ErrorCode::NotDefined,
            1 => ErrorCode::FileNotFound,
            2 => ErrorCode::AccessViolation,
            3 => ErrorCode::DiskFullOrAllocationFailed,
            4 => ErrorCode::IllegalTftpOperation,
            5 => ErrorCode::UnknownTransferId,
            6 => ErrorCode::FileAlreadyExists,
            7 => ErrorCode::NoSuchUser,
            _ => ErrorCode::NotDefined,
        }
    }
}

enum Packet {
    RRQ(String, String),
    WRQ(String, String),
    ACK(u16),
    ERROR(ErrorCode, String),
    Data(u16, Vec<u8>),
}

fn u8_array_to_vec(arr: &[u8]) -> Vec<u8> {
    arr.iter().cloned().collect()
}

fn main() {
    let local_addr = "127.0.0.1:6969";

    let socket = match UdpSocket::bind(local_addr) {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind socket: {}", e),
    };
    let mut buf = [0u8; MAX_PACKET_SIZE];

    println!("Waiting for UDP packet on port 127.0.0.1:6969");
    println!(" requesting /hello will return \"world\"");
    println!(" anything else returns File not found");
    match socket.recv_from(&mut buf) {
        Ok((amt, src)) => {
            println!("Got {} bytes from {}.", amt, src);
            match decode(&buf[..amt]) {
                Ok(Packet::RRQ(filename, mode_name)) => {
                    println!("RRQ opcode=1, filename={}, mode_name={}", filename, mode_name);

                    let out;

                    if filename == "hello" {
                        out = encode(Packet::Data(1, u8_array_to_vec("world".to_string().as_bytes()))).unwrap();
                    }
                    else {
                        out = encode(Packet::ERROR(ErrorCode::FileNotFound, "Test".to_string())).unwrap();
                    }


                    socket.send_to(&out[..], src).unwrap();
                },
                Ok(Packet::ERROR(error_code, error_msg)) => {
                    println!("ERR error_code={}, error_msg={}", error_code as u16, error_msg);
                },
                Ok(Packet::Data(block_no, data)) => {
                    println!("DATA opcode=3, block={}, data={} bytes", block_no, data.len());
                    unimplemented!();
                },
                Ok(Packet::WRQ(filename, mode_name)) => {
                    println!("WRQ opcode=2, filename={}, mode_name={}", filename, mode_name);
                    unimplemented!();
                },
                Ok(Packet::ACK(block_no)) => {
                    println!("ACL opcode=4, block_no={}", block_no);
                    unimplemented!();
                },
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        },
        Err(err) => println!("Can't recv_from: {}", err)
    }
}

fn encode(packet : Packet) -> Result<Vec<u8>, &'static str> {
    let mut buf : Vec<u8> = Vec::new();

    match packet {
        Packet::ERROR(error_code, error_string) => {
            if let Err(_) = buf.write_u16::<BigEndian>(5) {
                return Err("Error writing opcode")
            }
            if let Err(_) = buf.write_u16::<BigEndian>(error_code as u16) {
                return Err("Error writing error_code")
            }
            if let Err(_) = encode_string(&mut buf, error_string) {
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

fn decode(p : &[u8]) -> Result<Packet, String> {
    let mut reader = io::Cursor::new(p);

    let opcode_result = reader.read_u16::<byteorder::BigEndian>();

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
                    if let Ok(filename) = decode_string(&mut reader) {
                        if let Ok(mode_name) = decode_string(&mut reader) {
                            if opcode == 1 {
                                return Ok(Packet::RRQ(filename, mode_name))
                            }
                            else { //if opcode == 2
                                return Ok(Packet::WRQ(filename, mode_name))
                            }
                        }
                    }
                }
                5 => {
                    if let Ok(error_code) = reader.read_u16::<byteorder::BigEndian>() {
                        if let Ok(error_message) = decode_string(&mut reader) {
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

fn decode_string<T : byteorder::ReadBytesExt>(reader : &mut T) -> Result<String, &str> {
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
