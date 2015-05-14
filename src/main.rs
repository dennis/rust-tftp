extern crate byteorder;

use std::io;
use std::net::UdpSocket;
use std::str;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

const MAX_PACKET_SIZE : usize = 512;

// https://www.ietf.org/rfc/rfc1350.txt

enum Packet {
    RRQ(String, String),
    ERROR(u16, String),
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
                Some(Packet::RRQ(filename, mode_name)) => {
                    println!("RRQ opcode=1, filename={}, mode_name={}", filename, mode_name);

                    let out = encode(Packet::ERROR(1, "Test".to_string())).unwrap();

                    socket.send_to(&out[..], src);
                },
                Some(Packet::ERROR(error_code, error_msg)) => {
                    println!("ERR error_code={}, error_msg={}", error_code, error_msg);
                },
                None => {
                    println!("ERROR");
                }
            }
        },
        Err(err) => println!("Can't recv_from: {}", err)
    }
}

fn encode(packet : Packet) -> Option<Vec<u8>> {
    let mut buf : Vec<u8> = Vec::new();

    match packet {
        Packet::ERROR(error_code, error_string) => {
            // opcode
            encode_u16(&mut buf, 5);
            // error code
            encode_u16(&mut buf, error_code);
            // message
            encode_str(&mut buf, error_string);

            return Some(buf)
        },
        _ => {
            panic!("Unsupported");
        }
    }

    None
}

fn encode_u16(buf : &mut Vec<u8>, value : u16) {
    buf.push(((value >> 8) & 0xff) as u8);
    buf.push((value & 0xff) as u8);
}

fn encode_str(buf : &mut Vec<u8>, string : String) {
    for c in string.chars() {
        buf.push(c as u8);
    }
    buf.push(0);
}

fn decode(p : &[u8]) -> Option<Packet> {
    let mut reader = io::Cursor::new(p);

    let opcode_result = reader.read_u16::<byteorder::BigEndian>();

    match opcode_result {
        Ok(opcode) => {
            match opcode {
                // RRQ opcode 1
                1 => {
                    if let Some(filename) = decode_string(&mut reader) {
                        if let Some(mode_name) = decode_string(&mut reader) {
                            return Some(Packet::RRQ(filename, mode_name))
                        }
                    }
                }
                5 => {
                    if let Ok(error_code) = reader.read_u16::<byteorder::BigEndian>() {
                        if let Some(error_message) = decode_string(&mut reader) {
                            return Some(Packet::ERROR(error_code, error_message))
                        }
                    }
                },
                _ => {
                    println!("Unknown opcode");
                }
            }
        },
        Err(_) => {
            println!("Error");
        }
    }

    None
}

fn decode_string<T : byteorder::ReadBytesExt>(reader : &mut T) -> Option<String> {
    let mut string_bytes = Vec::new();

    while let Ok(c) = reader.read_u8() {
        if c == 0u8 {
            break;
        }

        string_bytes.push(c);
    }

    match str::from_utf8(&string_bytes[..]) {
        Ok(str) => Some(str.to_string()),
        Err(_) => None,
    }
}
