use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use time::{SteadyTime, Duration};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fs::File;

use packet::{Packet, ErrorCode};
use protocol::Protocol;
use stream::{TftpReadStream, TftpWriteStream, NullStream, FileStream, StringStream};

// Largest packet is DATA, which consist of 2 bytes opcode, 2 bytes block_no and up to 512 bytes
// data.. 
const MAX_PACKET_SIZE : usize = 2+2+512;
const PEER_REAPER_CHECK_SEC : i64 = 10;
const PEER_MAX_AGE_SEC : i64 = 300;

// https://www.ietf.org/rfc/rfc1350.txt

struct Peer<'a> {
    last_block_no : u16,
    last_activity : ::time::SteadyTime,
    read_stream : Box<TftpReadStream>,
    write_stream : Box<TftpWriteStream>,
    peer_socket: SocketAddr,
    source_socket: &'a UdpSocket,
    buffer : Vec<u8>,
    dead : bool,
}

impl<'a> Peer<'a> {
    #[allow(unused_must_use)]
    fn send_packet(&self, packet : Packet) {
        if let Ok(out) = Protocol::encode(packet) {
            println!("Sending {} bytes", out.len());

            // if we cannot send it, we just silently ignore it. The peer will
            // eventually get expired anyway
            self.source_socket.send_to(&out[..], self.peer_socket);
        }
        else {
            println!("Cannot encode packet!");
        }
    }
}

pub fn wip_server(local_addr : &str) {
    let socket = match UdpSocket::bind(local_addr) {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind socket: {}", e),
    };
    let mut buf = [0u8; MAX_PACKET_SIZE];
    let mut peers = HashMap::new();

    println!("Waiting for UDP packet on port 127.0.0.1:6969");

    let mut last_peer_reaper_check = SteadyTime::now();

    loop {
        let input = socket.recv_from(&mut buf);
        let now = SteadyTime::now();

        match input {
            Ok((amt, src)) => {
                println!("Got {} bytes from {}.", amt, src);

                match peers.entry(src) {
                    Vacant(entry) => {
                        entry.insert(Peer{
                            last_activity: now,
                            last_block_no: 0,
                            read_stream: Box::new(NullStream),
                            write_stream: Box::new(NullStream),
                            buffer : Vec::new(),
                            peer_socket: src,
                            source_socket: &socket,
                            dead : false,
                        });
                    },
                    Occupied(entry) => {
                        let peer = entry.into_mut();
                        peer.last_activity = now;
                    }
                }

                match peers.entry(src) {
                    Vacant(_) => {
                        // We have just added it, so this shouldn't be possible
                        // No peer found
                        println!("Peer not found");
                    },
                    Occupied(entry) => {
                        let mut peer = entry.into_mut();
                        peer.last_activity = now;

                        match Protocol::decode(&buf[..amt]) {
                            Ok(Packet::RRQ(filename, mode_name)) => {
                                handle_rrq(&mut peer, filename, mode_name);
                            },
                            Ok(Packet::ERROR(error_code, error_msg)) => {
                                handle_error(&mut peer, error_code, error_msg);
                            },
                            Ok(Packet::Data(block_no, data)) => {
                                handle_data(&mut peer, block_no, data);
                            },
                            Ok(Packet::WRQ(filename, mode_name)) => {
                                handle_wrq(&mut peer, filename, mode_name);
                            },
                            Ok(Packet::ACK(block_no)) => {
                                handle_ack(&mut peer, block_no);
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

        if now - last_peer_reaper_check > Duration::seconds(PEER_REAPER_CHECK_SEC) {
            let mut deletion : Vec<SocketAddr> = Vec::new();
            for pair in peers.iter() {
                let (socket, peer) = pair;
                if peer.dead || now - peer.last_activity > Duration::seconds(PEER_MAX_AGE_SEC) {
                    deletion.push(socket.clone());
                }
            }
            for socket in deletion {
                peers.remove(&socket);
            }

            last_peer_reaper_check = now;
        }
    }
}

fn handle_rrq(peer : &mut Peer, filename : String, mode_name : String) {
    println!("RRQ opcode=1, filename={}, mode_name={}", filename, mode_name);

    match handle_file_read(filename) {
        Ok(stream) => {
            peer.read_stream = stream;
            send_data_block(peer, 1);
        },
        Err(error) => {
            // File not found
            peer.send_packet(Packet::ERROR(ErrorCode::FileNotFound, error))
        }
    }
}

fn handle_wrq(peer : &mut Peer, filename : String, mode_name : String) {
    println!("WRQ opcode=2, filename={}, mode_name={}", filename, mode_name);

    match File::create(filename) {
        Ok(file) => {
            peer.write_stream = Box::new(FileStream::new(file));
            peer.send_packet(Packet::ACK(0));
        },
        Err(err) => {
            println!("Error: {}", err);
            peer.send_packet(Packet::ERROR(ErrorCode::NotDefined, err.to_string()))
        }
    }
}

fn send_data_block(peer : &mut Peer, block_no : u16) {
    let start : usize = ((block_no - 1) as usize) * MAX_PACKET_SIZE;
    let length : usize = MAX_PACKET_SIZE;

    if let Ok(bytes) = peer.read_stream.get_block(start, length) {
        if bytes.len() > 0 {
            println!("  Send data block: start={}, length={}", start, bytes.len());
            peer.buffer = bytes.clone();
            peer.last_block_no = block_no;
            peer.send_packet(Packet::Data(block_no, Box::new(bytes)));
        }
    }
    else {
        peer.send_packet(Packet::ERROR(ErrorCode::NotDefined, "I/O error eading block".to_string()))
    }
}

fn handle_ack(peer : &mut Peer, block_no : u16) {
    println!("ACK opcode=4, block_no={}, expected={}", block_no, peer.last_block_no);

    if block_no == peer.last_block_no {
        println!("  Sending next block {}", block_no + 1);
        send_data_block(peer, block_no + 1);
    }
    else {
        peer.send_packet(Packet::ERROR(ErrorCode::UnknownTransferId, format!("expected={}, got={}", block_no, peer.last_block_no)));
    }
}

fn handle_file_read(filename : String) -> Result<Box<TftpReadStream>, String> {
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
                Err(err.to_string())
            }
        }
    }
}

fn handle_data(peer : &mut Peer, block_no : u16, data : Box<Vec<u8>>) {
    println!("DATA opcode=3, block={}, data={} bytes", block_no, data.len());
    if block_no == peer.last_block_no {
        // send ack, we'have already sent this
        println!("  data: already got block: {}", block_no);
        peer.send_packet(Packet::ACK(block_no));
    }
    else if block_no == peer.last_block_no + 1 {
        // store this
        println!("  new data block: {}. {} bytes", block_no, data.len());
        if let Ok(_) = peer.write_stream.add_block(data) {
            peer.last_block_no = block_no;
            peer.send_packet(Packet::ACK(block_no));
        }
        else {
            println!("  new data block: {}, but I/O error", block_no);
            peer.send_packet(Packet::ERROR(ErrorCode::DiskFullOrAllocationFailed, "I/O error".to_string()));
        }
    }
    else {
        println!("  new data block: expected block {}, actual {}", peer.last_block_no+1, block_no);
        peer.send_packet(Packet::ERROR(ErrorCode::UnknownTransferId, format!("expected={}, got={}", block_no, peer.last_block_no)));
    }
}

fn handle_error(peer : &mut Peer, error_code : ErrorCode, error_msg : String) {
    println!("ERR error_code={}, error_msg={}", error_code as u16, error_msg);
    peer.dead = true;
}
