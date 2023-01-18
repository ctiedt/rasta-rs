pub mod scip;

pub(crate) fn str_to_sci_name(name: &str) -> [u8; 20] {
    let mut new_name = ['_' as u8; 20];
    if name.len() < 20 {
        new_name[..name.len()].clone_from_slice(name.as_bytes());
    } else {
        new_name[..20].clone_from_slice(&name.as_bytes()[..20])
    }
    new_name
}

pub enum ProtocolType {
    SCIProtocolP = 0x40,
    SCIProtocolLS = 0x30,
}

pub enum SCIMessageType {
    SCIPMessageTypeChangeLocation = 0x0001,
    SCIPMessageTypeLocationStatus = 0x000B,
    SCIPMessageTypeTimeout = 0x000C,
}

pub struct SCIPayload {
    data: [u8; 85],
    used: usize,
}

impl Default for SCIPayload {
    fn default() -> Self {
        Self {
            data: [0; 85],
            used: 0,
        }
    }
}

impl SCIPayload {
    fn from_slice(data: &[u8]) -> Self {
        let mut payload = Self::default();
        payload.used = data.len();
        payload.data.copy_from_slice(data);
        payload
    }
}

pub struct SCITelegram {
    protocol_type: ProtocolType,
    message_type: SCIMessageType,
    sender: [u8; 20],
    receiver: [u8; 20],
    payload: SCIPayload,
}
