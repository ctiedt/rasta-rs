use std::{collections::HashMap, net::SocketAddr};

use rasta_rs::{
    sci::{
        scip::{SCIPointLocation, SCIPointTargetLocation},
        SCICommand, SCIConnection, SCIMessageType, SCITelegram,
    },
    RastaConnection,
};

fn main() {
    let addr: SocketAddr = "127.0.0.1:8888".parse().unwrap();
    let conn = RastaConnection::try_new(addr, 42).unwrap();
    let sci_name_rasta_id_mapping = HashMap::from([("C".to_string(), 42), ("S".to_string(), 1337)]);
    let mut sender =
        SCIConnection::try_new(conn, "C".to_string(), sci_name_rasta_id_mapping).unwrap();
    let mut next_direction = SCIPointTargetLocation::PointLocationChangeToLeft;
    sender
        .run("S", |data| {
            if let Some(data) = data {
                dbg!(data.message_type);
                if data.message_type == SCIMessageType::scip_location_status() {
                    let location = SCIPointLocation::try_from(data.payload.data[0]).unwrap();
                    println!("Point is now at {location:?}");
                    next_direction = if location == SCIPointLocation::PointLocationLeft {
                        SCIPointTargetLocation::PointLocationChangeToRight
                    } else {
                        SCIPointTargetLocation::PointLocationChangeToLeft
                    };
                    SCICommand::Telegram(SCITelegram::change_location(
                        "C",
                        "S",
                        SCIPointTargetLocation::PointLocationChangeToLeft,
                    ));
                }
            }
            SCICommand::Telegram(SCITelegram::change_location("C", "S", next_direction))
        })
        .unwrap();
}
