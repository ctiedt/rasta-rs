use crate::RastaError;

pub mod scip;

pub(crate) fn str_to_sci_name(name: &str) -> Vec<u8> {
    let mut new_name = vec!['_' as u8; 20];
    if name.len() < 20 {
        new_name[..name.len()].clone_from_slice(name.as_bytes());
    } else {
        new_name[..20].clone_from_slice(&name.as_bytes()[..20])
    }
    new_name
}

#[repr(u8)]
pub enum ProtocolType {
    SCIProtocolP = 0x40,
    SCIProtocolLS = 0x30,
}

impl TryFrom<u8> for ProtocolType {
    type Error = RastaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x40 => Ok(Self::SCIProtocolP),
            0x30 => Ok(Self::SCIProtocolLS),
            v => Err(RastaError::Other(format!("Unknown SCI protocol `{v}`"))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SCIMessageType {
    SCIPMessageTypeChangeLocation = 0x0001,
    SCIPMessageTypeLocationStatus = 0x000B,
    SCIPMessageTypeTimeout = 0x000C,
}

impl TryFrom<u8> for SCIMessageType {
    type Error = RastaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0001 => Ok(Self::SCIPMessageTypeChangeLocation),
            0x000B => Ok(Self::SCIPMessageTypeLocationStatus),
            0x000C => Ok(Self::SCIPMessageTypeTimeout),
            v => Err(RastaError::Other(format!("Unknown SCI message `{v}`"))),
        }
    }
}

pub struct SCIPayload {
    pub data: [u8; 85],
    pub used: usize,
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
        payload.data[..data.len()].copy_from_slice(data);
        payload
    }
}

pub struct SCITelegram {
    pub protocol_type: ProtocolType,
    pub message_type: SCIMessageType,
    pub sender: String,
    pub receiver: String,
    pub payload: SCIPayload,
}

impl TryFrom<&[u8]> for SCITelegram {
    type Error = RastaError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self {
            protocol_type: ProtocolType::try_from(value[0])?,
            message_type: SCIMessageType::try_from(value[1])?,
            sender: String::from_utf8_lossy(&value[2..22]).to_string(),
            receiver: String::from_utf8_lossy(&value[22..42]).to_string(),
            payload: SCIPayload::from_slice(&value[42..]),
        })
    }
}

impl Into<Vec<u8>> for SCITelegram {
    fn into(self) -> Vec<u8> {
        let mut data = vec![self.protocol_type as u8, self.message_type as u8];
        data.append(&mut str_to_sci_name(&self.sender));
        data.append(&mut str_to_sci_name(&self.receiver));
        if self.payload.used > 0 {
            let mut payload = Vec::from(self.payload.data);
            data.append(&mut payload);
        }
        data
    }
}

pub enum SCICommand {
    Telegram(SCITelegram),
    Wait,
    Disconnect,
}
