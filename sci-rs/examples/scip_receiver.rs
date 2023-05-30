use std::net::SocketAddr;

use rasta_rs::RastaListener;
use sci_rs::{
    scip::{SCIPointLocation, SCIPointTargetLocation},
    SCIListener, SCIMessageType, SCITelegram,
};

fn main() {
    let addr: SocketAddr = "127.0.0.1:8888".parse().unwrap();
    let listener = RastaListener::try_new(addr, 1337).unwrap();

    let mut receiver = SCIListener::new(listener, "S".to_string());
    let mut location = SCIPointLocation::PointLocationLeft;
    receiver
        .listen(|telegram| {
            println!(
                "Received Telegram: {}",
                telegram.message_type.try_as_scip_message_type().unwrap()
            );
            dbg!(telegram.sender);
            dbg!(telegram.receiver);
            dbg!(telegram.payload.used);
            if telegram.message_type == SCIMessageType::scip_change_location() {
                let change = SCIPointTargetLocation::try_from(telegram.payload.data[0]).unwrap();
                match change {
                    SCIPointTargetLocation::PointLocationChangeToRight => {
                        location = SCIPointLocation::PointLocationRight
                    }
                    SCIPointTargetLocation::PointLocationChangeToLeft => {
                        location = SCIPointLocation::PointLocationLeft
                    }
                }
                Some(SCITelegram::location_status("S", "C", location))
            } else {
                None
            }
        })
        .unwrap();
}
