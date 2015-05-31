use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use time::{SteadyTime, Duration};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fs::File;

use packet::{Packet, ErrorCode};
use protocol::Protocol;
use stream::{TftpReadStream, NullStream, FileStream, StringStream};

const MAX_PACKET_SIZE : usize = 512;
const SESSION_REAPER_CHECK_SEC : i64 = 10;
const SESSION_MAX_AGE_SEC : i64 = 300;

// https://www.ietf.org/rfc/rfc1350.txt

struct Session {
    last_sent_block_no : u16,
    last_activity : ::time::SteadyTime,
    send_stream : Box<TftpReadStream>,
    buffer : Vec<u8>,
}

fn send_packet(socket : &UdpSocket, peer : &SocketAddr, packet : Packet) {
    let out = Protocol::encode(packet).unwrap();
    println!("Sending {} bytes", out.len());

    // if we cannot send it, we just silently ignore it. The session will
    // eventually get expired anyway
    socket.send_to(&out[..], peer).unwrap();
}

pub fn wip_server(local_addr : &str) {
    let socket = match UdpSocket::bind(local_addr) {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind socket: {}", e),
    };
    let mut buf = [0u8; MAX_PACKET_SIZE];
    let mut sessions = HashMap::new();

    println!("Waiting for UDP packet on port 127.0.0.1:6969");

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
                            last_sent_block_no: 0,
                            send_stream: Box::new(NullStream),
                            buffer : Vec::new()
                        });
                    },
                    Occupied(entry) => {
                        let session = entry.into_mut();
                        session.last_activity = now;
                    }
                }

                match sessions.entry(src) {
                    Vacant(_) => {
                        // We have just added it, so this shouldn't be possible
                        // No session found
                        send_packet(&socket, &src, Packet::ERROR(ErrorCode::UnknownTransferId, "".to_string()));
                    },
                    Occupied(entry) => {
                        let mut session = entry.into_mut();
                        session.last_activity = now;

                        match Protocol::decode(&buf[..amt]) {
                            Ok(Packet::RRQ(filename, mode_name)) => {
                                handle_rrq(&mut session, &socket, &src, filename, mode_name);
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
                                handle_ack(&mut session, &socket, &src, block_no);
                            },
                            Err(err) => {
                                println!("Error: {}", err);
                            }
                        }
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

fn handle_rrq(session : &mut Session, socket : &UdpSocket, src : &SocketAddr, filename : String, mode_name : String) {
    println!("RRQ opcode=1, filename={}, mode_name={}", filename, mode_name);

    match handle_file_read(filename) {
        Ok(stream) => {
            session.send_stream = stream;
            send_data_block(session, &socket, &src, 1);
        },
        Err(_) => {
            // File not found
            send_packet(&socket, &src, Packet::ERROR(ErrorCode::FileNotFound, "Test".to_string()))
        }
    }
}

fn send_data_block(session : &mut Session, socket : &UdpSocket, src : &SocketAddr, block_no : u16) {
    let start : usize = ((block_no - 1) as usize) * MAX_PACKET_SIZE;
    let length : usize = MAX_PACKET_SIZE;

    println!("  Send data block: start={}, length={}", start, length);

    if let Ok(bytes) = session.send_stream.get_block(start, length) {
        session.buffer = bytes.clone();
        session.last_sent_block_no = block_no;
        send_packet(&socket, &src, Packet::Data(block_no, &bytes));
    }
    else {
        send_packet(&socket, &src, Packet::ERROR(ErrorCode::NotDefined, "I/O error eading block".to_string()))
    }
}

fn handle_ack(session : &mut Session, socket : &UdpSocket, src : &SocketAddr, block_no : u16) {
    println!("ACK opcode=4, block_no={}, expected={}", block_no, session.last_sent_block_no);

    // FIXME we might send out block that does not exists
    if block_no == session.last_sent_block_no {
        println!("  Sending next block {}", block_no + 1);
        send_data_block(session, &socket, &src, block_no + 1);
    }
    else {
        send_packet(socket, src, Packet::ERROR(ErrorCode::UnknownTransferId, format!("expected={}, got={}", block_no, session.last_sent_block_no)));
    }
}

fn handle_file_read(filename : String) -> Result<Box<TftpReadStream>, ()> {
    if filename == "hello" {
        Ok(Box::new(StringStream::new("world".to_string())))
    }
    else {
        match File::open(filename) {
            Ok(file) => {
                Ok(Box::new(FileStream::new(file)))
            },
            Err(err) => {
                println!("Error: {}", err);
                Err(())
            }
        }
    }
}
