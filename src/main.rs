extern crate byteorder;
extern crate time;

mod packet;
mod protocol;
mod tftp;

fn main() {
    tftp::wip_server();
}
