extern crate byteorder;
extern crate time;

mod packet;
mod protocol;
mod tftp;

fn main() {
    tftp::wip_server("127.0.0.1:6969");
}
