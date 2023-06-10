use std::net::SocketAddr;
use rasta_rs::RastaListener;
use sci_rs::{SCIListener, SCIMessageType, SCITelegram};
use sci_rs::scils::SCILSBrightness;

fn main() {
    let addr: SocketAddr = "127.0.0.1:8888".parse().unwrap();
    let listener = RastaListener::try_new(addr, 1337).unwrap();

    let mut receiver = SCIListener::new(listener, "S".to_string());
    let mut luminosity = SCILSBrightness::Night;

    receiver.listen(|telegram| {
        println!(
            "Received Telegram: {}",
            telegram.message_type.try_as_scils_message_type().unwrap()
        );
        dbg!(&telegram.sender);
        dbg!(&telegram.receiver);
        dbg!(telegram.payload.used);
        if telegram.message_type == SCIMessageType::scils_change_brightness() {
            let change =
                SCILSBrightness::try_from(telegram.payload.data[0]).unwrap();
            luminosity = change;
            Some(SCITelegram::scils_brightness_status(&*telegram.receiver, &*telegram.sender, luminosity))
        } else { None}
    })
        .unwrap();
}