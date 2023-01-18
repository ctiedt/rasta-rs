use std::net::SocketAddrV4;

use rasta_rs::{RastaCommand, RastaConnection};

fn main() {
    let addr: SocketAddrV4 = "127.0.0.1:8888".parse().unwrap();
    let mut conn = RastaConnection::try_new(addr, 1234).unwrap();
    let mut sent = false;
    conn.run(5678, || {
        if !sent {
            sent = true;
            RastaCommand::Data(vec![1, 2, 3, 4])
        } else {
            RastaCommand::Wait
        }
    })
    .unwrap();
}
