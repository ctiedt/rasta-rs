use crate::{RastaConnection, RastaListener};

use super::{str_to_sci_name, ProtocolType, SCIMessageType, SCIPayload, SCITelegram};

#[repr(u8)]
pub enum SCIPointTargetLocation {
    PointLocationChangeToRight = 0x01,
    PointLocationChangeToLeft = 0x02,
}

pub enum SCIPointLocation {
    PointLocationRight = 0x01,
    PointLocationLeft = 0x02,
    PointNoTargetLocation = 0x03,
    PointBumped = 0x04,
}

impl SCITelegram {
    pub fn change_location(sender: &str, receiver: &str, to: SCIPointTargetLocation) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolP,
            message_type: SCIMessageType::SCIPMessageTypeChangeLocation,
            sender: str_to_sci_name(sender),
            receiver: str_to_sci_name(receiver),
            payload: SCIPayload::from_slice(&[to as u8]),
        }
    }
}

pub struct SCIPListener {
    listener: RastaListener,
}

pub struct SCIPConnection {
    conn: RastaConnection,
}
impl SCIPConnection {}
