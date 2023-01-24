use crate::RastaError;

pub mod scip;

pub const SCI_VERSION: u8 = 0x01;

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
    VersionRequest = 0x0024,
    VersionResponse = 0x0025,
    StatusRequest = 0x0021,
    StatusBegin = 0x0022,
    StatusFinish = 0x0023,
    ChangeLocation = 0x0001,
    LocationStatus = 0x000B,
    Timeout = 0x000C,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SCIVersionCheckResult {
    NotAllowedToUse = 0,
    VersionsAreNotEqual = 1,
    VersionsAreEqual = 2,
}

impl TryFrom<u8> for SCIVersionCheckResult {
    type Error = RastaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NotAllowedToUse),
            1 => Ok(Self::VersionsAreEqual),
            2 => Ok(Self::VersionsAreEqual),
            v => Err(RastaError::Other(format!(
                "Unknown SCI Version check result `{v:x}`"
            ))),
        }
    }
}

impl TryFrom<u8> for SCIMessageType {
    type Error = RastaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0001 => Ok(Self::ChangeLocation),
            0x000B => Ok(Self::LocationStatus),
            0x000C => Ok(Self::Timeout),
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

impl SCITelegram {
    pub fn version_request(
        protocol_type: ProtocolType,
        sender: &str,
        receiver: &str,
        version: u8,
    ) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::VersionRequest,
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[version]),
        }
    }

    pub fn version_response(
        protocol_type: ProtocolType,
        sender: &str,
        receiver: &str,
        version: u8,
        version_check_result: SCIVersionCheckResult,
        checksum: &[u8],
    ) -> Self {
        let mut payload_data = vec![version_check_result as u8, version, checksum.len() as u8];
        payload_data.append(&mut Vec::from(checksum));
        Self {
            protocol_type,
            message_type: SCIMessageType::VersionResponse,
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&payload_data),
        }
    }

    pub fn status_request(protocol_type: ProtocolType, sender: &str, receiver: &str) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::StatusRequest,
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }

    pub fn status_begin(protocol_type: ProtocolType, sender: &str, receiver: &str) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::StatusBegin,
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }

    pub fn status_finish(protocol_type: ProtocolType, sender: &str, receiver: &str) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::StatusFinish,
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }
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
