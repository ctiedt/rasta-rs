use std::collections::HashMap;
use std::net::SocketAddr;
use rasta_rs::RastaConnection;
use sci_rs::{SCICommand, SCIConnection, SCIMessageType, SCITelegram};
use sci_rs::scils::SCILSBrightness;

fn main() {
    let addr: SocketAddr = "127.0.0.1:8888".parse().unwrap();
    let conn = RastaConnection::try_new(addr,42).unwrap();
    let sci_name_rasta_id_mapping =
        HashMap::from([("C".to_string(), 42), ("S".to_string(), 1337)]);
    let mut sender =
        SCIConnection::try_new(conn,"C".to_string(),sci_name_rasta_id_mapping).unwrap();

    sender
        .run("S",|data| {
            SCICommand::Telegram(SCITelegram::scils_change_brightness(
                "C",
                "S",
                SCILSBrightness::Day,
            ))
        })
        .unwrap();
}