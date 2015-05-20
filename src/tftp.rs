use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use time::{SteadyTime, Duration};
use std::collections::hash_map::Entry::{Occupied, Vacant};

use packet::{Packet, ErrorCode};
use protocol::Protocol;

const MAX_PACKET_SIZE : usize = 512;
const SESSION_REAPER_CHECK_SEC : i64 = 10;
const SESSION_MAX_AGE_SEC : i64 = 300;

// https://www.ietf.org/rfc/rfc1350.txt

fn u8_array_to_vec(arr: &[u8]) -> Vec<u8> {
    arr.iter().cloned().collect()
}

struct Session {
    last_ack_block_no : u16,
    last_sent_block_no : u16,
    last_activity : ::time::SteadyTime,
}

pub fn wip_server(local_addr : &str) {
    let socket = match UdpSocket::bind(local_addr) {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind socket: {}", e),
    };
    let mut buf = [0u8; MAX_PACKET_SIZE];
    let mut sessions = HashMap::new();

    println!("Waiting for UDP packet on port 127.0.0.1:6969");
    println!(" requesting /hello will return \"world\"");
    println!(" anything else returns File not found");

    let mut last_session_reaper_check = SteadyTime::now();

    loop {
        let input = socket.recv_from(&mut buf);
        let now = SteadyTime::now();

        match input {
            Ok((amt, src)) => {
                println!("Got {} bytes from {}.", amt, src);

                match sessions.entry(src) {
                    Vacant(entry) => {
                        entry.insert(Session{
                            last_activity: now,
                            last_ack_block_no: 0,
                            last_sent_block_no: 0
                        });
                    },
                    Occupied(entry) => {
                        let mut session = entry.into_mut();
                        session.last_activity = now;
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
                        println!("ACK opcode=4, block_no={}", block_no);
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

        if now - last_session_reaper_check > Duration::seconds(SESSION_REAPER_CHECK_SEC) {
            let mut deletion : Vec<SocketAddr> = Vec::new();
            for pair in sessions.iter() {
                let (socket, session) = pair;
                if now - session.last_activity > Duration::seconds(SESSION_MAX_AGE_SEC) {
                    deletion.push(socket.clone());
                }
            }
            for socket in deletion {
                sessions.remove(&socket);
            }

            last_session_reaper_check = now;
        }
    }
}

