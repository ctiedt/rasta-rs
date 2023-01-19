use std::net::SocketAddrV4;

use rasta_rs::{message::Message, RastaListener};

fn on_receive(msg: Message) -> Option<Vec<u8>> {
    dbg!(msg.data());
    Some(vec![5, 6, 7, 8])
}

fn main() {
    let addr: SocketAddrV4 = "127.0.0.1:8888".parse().unwrap();
    let mut conn = RastaListener::try_new(addr, 1337).unwrap();
    conn.listen(on_receive).unwrap();
}
