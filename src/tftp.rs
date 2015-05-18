use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use time::SteadyTime;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use packet::{Packet, ErrorCode};
use protocol::Protocol;

const MAX_PACKET_SIZE : usize = 512;

// https://www.ietf.org/rfc/rfc1350.txt

fn u8_array_to_vec(arr: &[u8]) -> Vec<u8> {
    arr.iter().cloned().collect()
}

struct Session {
    last_ack_block_no : u16,
    last_sent_block_no : u16,
    last_activity : ::time::SteadyTime,
}

pub fn wip_server() {
    let local_addr = "127.0.0.1:6969";

    let socket = match UdpSocket::bind(local_addr) {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind socket: {}", e),
    };
    let mut buf = [0u8; MAX_PACKET_SIZE];
    let mut sessions = HashMap::new();

    println!("Waiting for UDP packet on port 127.0.0.1:6969");
    println!(" requesting /hello will return \"world\"");
    println!(" anything else returns File not found");

    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                println!("Got {} bytes from {}.", amt, src);

                // FIXME Expire old sessions

                match sessions.entry(src) {
                    Vacant(entry) => {
                        println!("New session");

                        entry.insert(Session{
                            last_activity: SteadyTime::now(),
                            last_ack_block_no: 0,
                            last_sent_block_no: 0
                        });
                    },
                    Occupied(entry) => {
                        println!("Known session");

                        let mut session = entry.into_mut();
                        session.last_activity = SteadyTime::now();
                    }
                }

                match Protocol::decode(&buf[..amt]) {
                    Ok(Packet::RRQ(filename, mode_name)) => {
                        println!("RRQ opcode=1, filename={}, mode_name={}", filename, mode_name);

                        let out;

                        if filename == "hello" {
                            out = Protocol::encode(Packet::Data(1, u8_array_to_vec("world".to_string().as_bytes()))).unwrap();
                        }
                        else {
                            out = Protocol::encode(Packet::ERROR(ErrorCode::FileNotFound, "Test".to_string())).unwrap();
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
            Err(err) => { 
                println!("Can't recv_from: {}", err);
                break;
            }
        }
    }
}

