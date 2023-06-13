use rasta_rs::RastaConnection;
use sci_rs::scils::SCILSBrightness;
use sci_rs::{SCICommand, SCIConnection, SCITelegram};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

fn main() {
    let addr: SocketAddr = "127.0.0.1:8888".parse().unwrap();
    let conn = RastaConnection::try_new(addr, 42).unwrap();
    let sci_name_rasta_id_mapping = HashMap::from([("C".to_string(), 42), ("S".to_string(), 1337)]);
    let mut sender =
        SCIConnection::try_new(conn, "C".to_string(), sci_name_rasta_id_mapping).unwrap();

    let target_luminosity = SCILSBrightness::Day;
    let mut current_luminosity = SCILSBrightness::Night;
    let lock = RwLock::new(target_luminosity);
    let send_lock = Arc::new(lock);
    let input_lock = send_lock.clone();

    thread::spawn(move || loop {
        {
            let mut locked_luminosity = input_lock.write().unwrap();
            *locked_luminosity = if *locked_luminosity == SCILSBrightness::Day {
                SCILSBrightness::Night
            } else {
                SCILSBrightness::Day
            };
            println!("ts_input: {:?} ", *locked_luminosity);
        }

        thread::sleep(Duration::from_millis(1000));
    });

    sender
        .run("S", |_data| {
            let locked_luminosity = send_lock.read().unwrap();
            println!("ts_sending: {:?} ", locked_luminosity);
            if current_luminosity != *locked_luminosity {
                println!("sending telegram now");
                current_luminosity = *locked_luminosity;
                return SCICommand::Telegram(SCITelegram::scils_change_brightness(
                    "C",
                    "S",
                    *locked_luminosity,
                ));
            }
            SCICommand::Wait
        })
        .unwrap();

    println!("Getting here ?");
}
