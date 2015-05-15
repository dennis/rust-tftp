extern crate byteorder;

use std::io;
use std::net::UdpSocket;
use std::str;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

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
    ERROR(ErrorCode, String),
}

fn main() {
    let local_addr = "127.0.0.1:6969";

    let socket = match UdpSocket::bind(local_addr) {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind socket: {}", e),
    };
    let mut buf = [0u8; MAX_PACKET_SIZE];

    println!("Waiting for UDP packet on port 127.0.0.1:6969");
    match socket.recv_from(&mut buf) {
        Ok((amt, src)) => {
            println!("Got {} bytes from {}.", amt, src);
            match decode(&buf[..amt]) {
                Ok(Packet::RRQ(filename, mode_name)) => {
                    println!("RRQ opcode=1, filename={}, mode_name={}", filename, mode_name);

                    let out = encode(Packet::ERROR(ErrorCode::FileNotFound, "Test".to_string())).unwrap();

                    socket.send_to(&out[..], src).unwrap();
                },
                Ok(Packet::ERROR(error_code, error_msg)) => {
                    println!("ERR error_code={}, error_msg={}", error_code as u16, error_msg);
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
            buf.write_u16::<BigEndian>(5).unwrap(); // opcode
            buf.write_u16::<BigEndian>(error_code as u16).unwrap(); // error code
            encode_string(&mut buf, error_string); // message

            return Ok(buf)
        },
        _ => {
            Err("Unsuported packet to encode")
        }
    }
}

fn encode_string(buf : &mut Vec<u8>, string : String) {
    for c in string.chars() {
        buf.write_u8(c as u8).unwrap();
    }
    buf.write_u8(0u8).unwrap();
}

fn decode(p : &[u8]) -> Result<Packet, &str> {
    let mut reader = io::Cursor::new(p);

    let opcode_result = reader.read_u16::<byteorder::BigEndian>();

    match opcode_result {
        Ok(opcode) => {
            match opcode {
                // RRQ opcode 1
                1 => {
                    if let Ok(filename) = decode_string(&mut reader) {
                        if let Ok(mode_name) = decode_string(&mut reader) {
                            return Ok(Packet::RRQ(filename, mode_name))
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
                _ => {
                    return Err("Unknown opcode")
                }
            }
        },
        Err(_) => {
            return Err("Error decoding opcode")
        }
    }

    Err("Error parsing packet")
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
