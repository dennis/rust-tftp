use std::net::UdpSocket;

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

fn decode(p : &[u8]) {
    let mut iter = p.iter();
    let mut filename = "".to_string();
    let mut mode_name = "".to_string();

    // opcode
    let opcode = decode_u16(*iter.next().unwrap(), *iter.next().unwrap());
    println!("opcode: {}", opcode);

    // filename
    while let Some(x) = iter.next() {
        if *x == 0 {
            break;
        }
        else {
            let ch = std::char::from_u32(*x as u32).unwrap();
            filename.push(ch);
        }
    }
    println!("filename: {}", filename);

    // mode
    while let Some(x) = iter.next() {
        if *x == 0 {
            break;
        }
        else {
            let ch = std::char::from_u32(*x as u32).unwrap();
            mode_name.push(ch);
        }
    }
    println!("mode_name: {}", mode_name);
}

fn decode_u16(high : u8, lower : u8) -> u16 {
    let mut result : u16 = high as u16;

    result = result << 8;
    result = result + (lower as u16);

    result
}

