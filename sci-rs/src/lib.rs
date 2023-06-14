//! # SCI Protocols
//!
//! SCI is the family of application protocols built on top of RaSTA
//! to communicate with track elements such as points and signals.
//! `rasta-rs` provides support for SCI-P at the moment.

#[cfg(feature = "rasta")]
use std::collections::HashMap;

#[cfg(feature = "rasta")]
use rasta_rs::{
    message::RastaId, RastaConnection, RastaConnectionState, RastaError, RastaListener,
    RASTA_TIMEOUT_DURATION,
};
use scils::SciLsError;
use scip::SciPError;

#[derive(Debug, Clone)]
pub enum SciError {
    UnknownProtocol(u8),
    UnknownMessageType(u8),
    UnknownVersionCheckResult(u8),
    Ls(SciLsError),
    P(SciPError),
}

impl From<SciLsError> for SciError {
    fn from(value: SciLsError) -> Self {
        SciError::Ls(value)
    }
}

impl From<SciPError> for SciError {
    fn from(value: SciPError) -> Self {
        SciError::P(value)
    }
}

#[cfg(feature = "rasta")]
impl From<SciError> for RastaError {
    fn from(value: SciError) -> Self {
        Self::Other(format!("{:?}", value))
    }
}

pub mod scils;
pub mod scip;

/// The current version of this SCI implementation.
pub const SCI_VERSION: u8 = 0x01;

pub(crate) fn str_to_sci_name(name: &str) -> Vec<u8> {
    let mut new_name = vec![b'_'; 20];
    if name.len() < 20 {
        new_name[..name.len()].clone_from_slice(name.as_bytes());
    } else {
        new_name[..20].clone_from_slice(&name.as_bytes()[..20])
    }
    new_name
}

/// Constants to represent SCI Protocol types.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ProtocolType {
    SCIProtocolP = 0x40,
    SCIProtocolLS = 0x30,
}

impl TryFrom<u8> for ProtocolType {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x40 => Ok(Self::SCIProtocolP),
            0x30 => Ok(Self::SCIProtocolLS),
            v => Err(SciError::UnknownProtocol(v)),
        }
    }
}

/// The message types for SCI messages. Since
/// protocols may use overlapping integer
/// representations, this is not a enum, but a
/// newtype with associated functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SCIMessageType(u8);

impl SCIMessageType {
    pub const fn sci_version_request() -> Self {
        Self(0x0024)
    }

    pub const fn sci_version_response() -> Self {
        Self(0x0025)
    }

    pub const fn sci_status_request() -> Self {
        Self(0x0021)
    }

    pub const fn sci_status_begin() -> Self {
        Self(0x0022)
    }

    pub const fn sci_status_finish() -> Self {
        Self(0x0023)
    }

    pub const fn sci_timeout() -> Self {
        Self(0x000C)
    }

    pub fn try_as_sci_message_type(&self) -> Result<&str, SciError> {
        match self.0 {
            0x0024 => Ok("VersionRequest"),
            0x0025 => Ok("VersionResponse"),
            0x0021 => Ok("StatusRequest"),
            0x0022 => Ok("StatusBegin"),
            0x0023 => Ok("StatusFinish"),
            0x000C => Ok("Timeout"),
            v => Err(SciError::UnknownMessageType(v)),
        }
    }

    pub fn try_as_scip_message_type(&self) -> Result<&str, SciError> {
        match self.0 {
            0x0001 => Ok("ChangeLocation"),
            0x000B => Ok("LocationStatus"),
            _ => self.try_as_sci_message_type(),
        }
    }

    pub fn try_as_scip_message_type_from(value: u8) -> Result<Self, SciError> {
        match value {
            0x0001 => Ok(Self::scip_change_location()),
            0x000B => Ok(Self::scip_location_status()),
            0x000C => Ok(Self::sci_timeout()),
            v => Err(SciError::UnknownMessageType(v)),
        }
    }

    pub fn try_as_scils_message_type(&self) -> Result<&str, SciError> {
        match self.0 {
            0x0001 => Ok("ShowSignalAspect"),
            0x0002 => Ok("ChangeBrightness"),
            0x0003 => Ok("SignalAspectStatus"),
            0x0004 => Ok("BrightnessStatus"),
            _ => self.try_as_sci_message_type(),
        }
    }

    pub fn try_as_scils_message_type_from(value: u8) -> Result<Self, SciError> {
        match value {
            0x0001 => Ok(Self::scils_show_signal_aspect()),
            0x0002 => Ok(Self::scils_change_brightness()),
            0x0003 => Ok(Self::scils_signal_aspect_status()),
            0x0004 => Ok(Self::scils_brightness_status()),
            0x000C => Ok(Self::sci_timeout()),
            v => Err(SciError::UnknownMessageType(v)),
        }
    }
}

impl From<SCIMessageType> for u8 {
    fn from(val: SCIMessageType) -> Self {
        val.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SCIVersionCheckResult {
    NotAllowedToUse = 0,
    VersionsAreNotEqual = 1,
    VersionsAreEqual = 2,
}

impl TryFrom<u8> for SCIVersionCheckResult {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NotAllowedToUse),
            1 => Ok(Self::VersionsAreEqual),
            2 => Ok(Self::VersionsAreEqual),
            v => Err(SciError::UnknownVersionCheckResult(v)),
        }
    }
}

/// The payload of an [`SCITelegram`]. Usually constructed from
/// a slice using [`SCIPayload::from_slice`].
#[derive(Clone, Copy)]
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
        let mut payload = Self {
            used: data.len(),
            ..Default::default()
        };
        payload.data[..data.len()].copy_from_slice(data);
        payload
    }
}

/// An SCI message. You should construct these using the generic
/// and protocol-specific associated functions.
#[derive(Clone)]
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
            message_type: SCIMessageType::sci_version_request(),
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
            message_type: SCIMessageType::sci_version_response(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&payload_data),
        }
    }

    pub fn status_request(protocol_type: ProtocolType, sender: &str, receiver: &str) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::sci_status_request(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }

    pub fn status_begin(protocol_type: ProtocolType, sender: &str, receiver: &str) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::sci_status_begin(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }

    pub fn status_finish(protocol_type: ProtocolType, sender: &str, receiver: &str) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::sci_status_finish(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }

    pub fn timeout(protocol_type: ProtocolType, sender: &str, receiver: &str) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::sci_timeout(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }
}

impl TryFrom<&[u8]> for SCITelegram {
    type Error = SciError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let protocol_type = ProtocolType::try_from(value[0])?;
        let message_type = match protocol_type {
            ProtocolType::SCIProtocolP => SCIMessageType::try_as_scip_message_type_from(value[1])?,
            ProtocolType::SCIProtocolLS => SCIMessageType::try_as_scils_message_type_from(value[1])?
        };
        Ok(Self {
            protocol_type,
            message_type,
            sender: String::from_utf8_lossy(&value[2..22]).to_string(),
            receiver: String::from_utf8_lossy(&value[22..42]).to_string(),
            payload: SCIPayload::from_slice(&value[42..]),
        })
    }
}

impl From<SCITelegram> for Vec<u8> {
    fn from(val: SCITelegram) -> Self {
        let mut data = vec![val.protocol_type as u8, val.message_type.into()];
        data.append(&mut str_to_sci_name(&val.sender));
        data.append(&mut str_to_sci_name(&val.receiver));
        if val.payload.used > 0 {
            let mut payload = Vec::from(val.payload.data);
            data.append(&mut payload);
        }
        data
    }
}

/// The SCI equivalent of [`crate::RastaCommand`].
#[cfg(feature = "rasta")]
#[derive(Clone)]
pub enum SCICommand {
    Telegram(SCITelegram),
    Wait,
    Disconnect,
}

/// A listening SCI endpoint built on top of [`RastaListener`].
/// [`SCIPListener::listen`] follows the same conventions as
/// [`RastaListener::listen`].
#[cfg(feature = "rasta")]
pub struct SCIListener {
    listener: RastaListener,
    name: String,
}

#[cfg(feature = "rasta")]
impl SCIListener {
    pub fn new(listener: RastaListener, name: String) -> Self {
        Self { listener, name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn listen<F>(&mut self, mut on_receive: F) -> Result<(), RastaError>
    where
        F: FnMut(SCITelegram) -> Option<SCITelegram>,
    {
        self.listener.listen(|data| {
            if let Some(response) = (on_receive)(SCITelegram::try_from(data.data()).unwrap()) {
                let data: Vec<u8> = response.into();
                Some(data)
            } else {
                None
            }
        })
    }
}

/// A sending SCI endpoint built on top of [`RastaConnection`].
/// [`SCIPConnection::run`] follows the same conventions as
/// [`RastaConnection::run`] but using the [`SCICommand`] type
/// for control flow.
#[cfg(feature = "rasta")]
pub struct SCIConnection {
    conn: RastaConnection,
    name: String,
    sci_name_rasta_id_mapping: HashMap<String, RastaId>,
}

#[cfg(feature = "rasta")]
impl SCIConnection {
    pub fn try_new(
        conn: RastaConnection,
        name: String,
        sci_name_rasta_id_mapping: HashMap<String, RastaId>,
    ) -> Result<Self, RastaError> {
        if conn.connection_state_request() == RastaConnectionState::Down {
            Ok(Self {
                conn,
                name,
                sci_name_rasta_id_mapping,
            })
        } else {
            Err(RastaError::StateError)
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn send_telegram(&mut self, telegram: SCITelegram) -> Result<(), RastaError> {
        if self.conn.connection_state_request() == RastaConnectionState::Down {
            let receiver = self
                .sci_name_rasta_id_mapping
                .get(&telegram.receiver)
                .ok_or(RastaError::Other("Missing Rasta ID".to_string()))?;
            self.conn.open_connection(*receiver)?;
        }
        let data: Vec<u8> = telegram.into();
        self.conn.send_data(data.as_slice())?;
        Ok(())
    }

    pub fn receive_telegram(&mut self) -> Result<SCITelegram, RastaError> {
        let msg = self.conn.receive_message()?;
        SCITelegram::try_from(msg.data()).map_err(|e| e.into())
    }

    pub fn run<F>(&mut self, peer: &str, mut telegram_fn: F) -> Result<(), RastaError>
    where
        F: FnMut(Option<SCITelegram>) -> SCICommand,
    {
        if self.conn.connection_state_request() == RastaConnectionState::Down {
            let receiver = self
                .sci_name_rasta_id_mapping
                .get(peer)
                .ok_or(RastaError::Other("Missing Rasta ID".to_string()))?;
            self.conn.open_connection(*receiver)?;
        }
        let mut previous_data = None;
        loop {
            match telegram_fn(previous_data.take()) {
                SCICommand::Telegram(telegram) => {
                    self.send_telegram(telegram)?;
                    let telegram = self.receive_telegram()?;
                    previous_data.replace(telegram);
                }
                SCICommand::Wait => {
                    self.conn.send_heartbeat()?;
                    std::thread::sleep(RASTA_TIMEOUT_DURATION / 2);
                }
                SCICommand::Disconnect => {
                    self.conn.close_connection()?;
                    break;
                }
            }
        }
        Ok(())
    }
}
