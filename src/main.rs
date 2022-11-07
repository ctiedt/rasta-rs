use rasta_rs::message::Message;

fn main() {
    let msg = Message::connection_request(0, 0, 0, 100);
    println!("{}", std::mem::size_of_val(&msg));
    let data: &[u8] = &msg;
    dbg!(data);
}
