use std::net::UdpSocket;
use std::iter::Iterator;

const MAX_PACKET_SIZE : usize = 512;

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
            decode(&buf[..amt]);
        },
        Err(err) => println!("Can't recv_from: {}", err)
    }
}

fn decode(p : & [u8]) {
    let mut iter = p.iter();

    let opcode    = decode_u16(&mut iter).unwrap();
    let filename  = decode_string(&mut iter).unwrap();
    let mode_name = decode_string(&mut iter).unwrap();

    println!("filename: {}", filename);
    println!("opcode: {}", opcode);
    println!("mode_name: {}", mode_name);
}

fn decode_u16<'a, I : Iterator<Item=&'a u8>>(iter : &mut I) -> Option<u16> {
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
