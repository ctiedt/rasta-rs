//! # SCI Protocols
//!
//! SCI is the family of application protocols built on top of RaSTA
//! to communicate with track elements such as points and signals.
//! `rasta-rs` provides support for SCI-P at the moment.

use std::collections::HashMap;

use crate::{
    message::RastaId, RastaConnection, RastaConnectionState, RastaError, RastaListener,
    RASTA_TIMEOUT_DURATION,
};

pub mod scils;
pub mod scip;

/// The current version of this SCI implementation.
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

/// Constants to represent SCI Protocol types.
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
}

impl Into<u8> for SCIMessageType {
    fn into(self) -> u8 {
        self.0
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
            0x0001 => Ok(Self::scip_change_location()),
            0x000B => Ok(Self::scip_location_status()),
            0x000C => Ok(Self::sci_timeout()),
            v => Err(RastaError::Other(format!("Unknown SCI message `{v}`"))),
        }
    }
}

/// The payload of an [`SCITelegram`]. Usually constructed from
/// a slice using [`SCIPayload::from_slice`].
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

/// An SCI message. You should construct these using the generic
/// and protocol-specific associated functions.
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
        let mut data = vec![self.protocol_type as u8, self.message_type.into()];
        data.append(&mut str_to_sci_name(&self.sender));
        data.append(&mut str_to_sci_name(&self.receiver));
        if self.payload.used > 0 {
            let mut payload = Vec::from(self.payload.data);
            data.append(&mut payload);
        }
        data
    }
}

/// The SCI equivalent of [`crate::RastaCommand`].
pub enum SCICommand {
    Telegram(SCITelegram),
    Wait,
    Disconnect,
}

/// A listening SCI endpoint built on top of [`RastaListener`].
/// [`SCIPListener::listen`] follows the same conventions as
/// [`RastaListener::listen`].
pub struct SCIListener {
    listener: RastaListener,
    name: String,
}

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
pub struct SCIConnection {
    conn: RastaConnection,
    name: String,
    sci_name_rasta_id_mapping: HashMap<String, RastaId>,
}
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
        SCITelegram::try_from(msg.data())
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
