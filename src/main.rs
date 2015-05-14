use std::net::UdpSocket;
use std::iter::Iterator;

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
                    println!("RRQ filename={}, mode_name={}", filename, mode_name);

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

fn decode(p : & [u8]) -> Option<Packet> {
    let mut iter = p.iter();

    match decode_u16(&mut iter) {
        // RRQ opcode 1
        Some(1) => {
            if let Some(filename) = decode_string(&mut iter) {
                if let Some(mode_name) = decode_string(&mut iter) {
                    return Some(Packet::RRQ(filename, mode_name))
                }
            }

            None
        },
        // ERROR opcode 5
        Some(5) => {
            if let Some(error_code) = decode_u16(&mut iter) {
                if let Some(error_message) = decode_string(&mut iter) {
                    return Some(Packet::ERROR(error_code, error_message))
                }
            }

            None
        },
        Some(_) | None => {
            None
        }
    }
}

fn decode_u16<'a, I : Iterator<Item=&'a u8>>(iter : &mut I) -> Option<u16> {
    // FIXME big/little endian support
    if let Some(i) = iter.next() {
        let mut r = *i as u16;
        r = r << 8;

        if let Some(j) = iter.next() {
            r = r + *j as u16;
            return Some(r)
        }
    }

    None
}

fn decode_string<'a, I : Iterator<Item=&'a u8>>(iter : &mut I) -> Option<String> {
    let mut string = "".to_string();

    while let Some(x) = iter.next() {
        if *x == 0 {
            return Some(string)
        }
        else {
            let ch = std::char::from_u32(*x as u32).unwrap();
            string.push(ch);
        }
    }

    None
}
