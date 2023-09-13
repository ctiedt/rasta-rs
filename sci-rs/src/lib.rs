//! # SCI Protocols
//!
//! SCI is the family of application protocols built on top of RaSTA
//! to communicate with track elements such as points and signals.
//! `rasta-rs` provides support for SCI-P at the moment.

#[cfg(feature = "rasta")]
use std::collections::HashMap;
use std::fmt::Display;

#[cfg(feature = "rasta")]
use rasta_rs::{
    message::RastaId, RastaConnection, RastaConnectionState, RastaError, RastaListener,
    RASTA_TIMEOUT_DURATION,
};
#[cfg(feature = "scils")]
use scils::SciLsError;
#[cfg(feature = "scip")]
use scip::SciPError;

#[derive(Debug, Clone)]
pub enum SciError {
    UnknownProtocol(u8),
    UnknownMessageType(u16),
    UnknownVersionCheckResult(u8),
    UnknownCloseReason(u8),
    #[cfg(feature = "scils")]
    Ls(SciLsError),
    #[cfg(feature = "scip")]
    P(SciPError),
}

impl Display for SciError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reason = match self {
            SciError::UnknownProtocol(p) => format!("Unknown Protocol {:x}", p),
            SciError::UnknownMessageType(m) => format!("Unknown Message Type {:x}", m),
            SciError::UnknownVersionCheckResult(v) => {
                format!("Unknown Version Check Result {:x}", v)
            }
            SciError::UnknownCloseReason(c) => format!("Unknown Close Reason {:x}", c),
            #[cfg(feature = "scils")]
            SciError::Ls(l) => l.to_string(),
            #[cfg(feature = "scip")]
            SciError::P(p) => p.to_string(),
        };
        write!(f, "{}", reason)
    }
}

impl std::error::Error for SciError {}

#[cfg(feature = "scils")]
impl From<SciLsError> for SciError {
    fn from(value: SciLsError) -> Self {
        SciError::Ls(value)
    }
}

#[cfg(feature = "scip")]
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

#[cfg(feature = "scils")]
pub mod scils;
#[cfg(feature = "scip")]
pub mod scip;
#[cfg(feature = "scitds")]
pub mod scitds;

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
    SCIProtocolAIS = 0x01,
    SCIProtocolTDS = 0x20,
    SCIProtocolLS = 0x30,
    SCIProtocolP = 0x40,
    SCIProtocolRBC = 0x50,
    SCIProtocolLX = 0x60,
    SCIProtocolTCS = 0x70,
    SCIProtocolGIO = 0x90,
    SCIProtocolELX = 0xC0,
}

impl TryFrom<u8> for ProtocolType {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x20 => Ok(Self::SCIProtocolTDS),
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
pub struct SCIMessageType(u16);

/// Automatically implement the associated functions for message types.
#[macro_export]
macro_rules! impl_sci_message_type {
    ($(($msg:tt, $id:tt)),*) => {
        impl SCIMessageType {
            $(pub const fn $msg() -> Self {
                Self($id)
            })*
        }
    };
}

impl_sci_message_type!(
    (pdi_version_check, 0x0024),
    (pdi_version_response, 0x0025),
    (pdi_initialisation_request, 0x0021),
    (pdi_initialisation_response, 0x0022),
    (pdi_initialisation_completed, 0x0023),
    (pdi_close, 0x0027),
    (pdi_release_for_maintenance, 0x0028),
    (pdi_available, 0x0029),
    (pdi_not_available, 0x002A),
    (pdi_reset, 0x002B),
    (sci_timeout, 0x000C)
);

impl SCIMessageType {
    pub fn try_as_sci_message_type(&self) -> Result<&str, SciError> {
        match self.0 {
            0x0024 => Ok("VersionRequest"),
            0x0025 => Ok("VersionResponse"),
            0x0021 => Ok("StatusRequest"),
            0x0022 => Ok("StatusBegin"),
            0x0023 => Ok("StatusFinish"),
            0x0027 => Ok("Close"),
            0x0028 => Ok("ReleaseForMaintenance"),
            0x0029 => Ok("Available"),
            0x002A => Ok("NotAvailable"),
            0x002B => Ok("Reset"),
            0x000C => Ok("Timeout"),
            v => Err(SciError::UnknownMessageType(v)),
        }
    }

    pub fn try_as_sci_message_type_from(value: u16) -> Result<Self, SciError> {
        match value {
            0x0024 => Ok(Self::pdi_version_check()),
            0x0025 => Ok(Self::pdi_version_response()),
            0x0021 => Ok(Self::pdi_initialisation_request()),
            0x0022 => Ok(Self::pdi_initialisation_response()),
            0x0023 => Ok(Self::pdi_initialisation_completed()),
            0x0027 => Ok(Self::pdi_close()),
            0x0028 => Ok(Self::pdi_release_for_maintenance()),
            0x0029 => Ok(Self::pdi_available()),
            0x002A => Ok(Self::pdi_not_available()),
            0x002B => Ok(Self::pdi_reset()),
            0x000C => Ok(Self::sci_timeout()),
            v => Err(SciError::UnknownMessageType(v)),
        }
    }

    #[cfg(feature = "scip")]
    pub fn try_as_scip_message_type(&self) -> Result<&str, SciError> {
        match self.0 {
            0x0001 => Ok("ChangeLocation"),
            0x000B => Ok("LocationStatus"),
            _ => self.try_as_sci_message_type(),
        }
    }

    #[cfg(feature = "scip")]
    pub fn try_as_scip_message_type_from(value: u16) -> Result<Self, SciError> {
        match value {
            0x0001 => Ok(Self::scip_change_location()),
            0x000B => Ok(Self::scip_location_status()),
            _ => Self::try_as_sci_message_type_from(value),
        }
    }

    #[cfg(feature = "scils")]
    pub fn try_as_scils_message_type(&self) -> Result<&str, SciError> {
        match self.0 {
            0x0001 => Ok("ShowSignalAspect"),
            0x0002 => Ok("ChangeBrightness"),
            0x0003 => Ok("SignalAspectStatus"),
            0x0004 => Ok("BrightnessStatus"),
            _ => self.try_as_sci_message_type(),
        }
    }

    #[cfg(feature = "scils")]
    pub fn try_as_scils_message_type_from(value: u16) -> Result<Self, SciError> {
        match value {
            0x0001 => Ok(Self::scils_show_signal_aspect()),
            0x0002 => Ok(Self::scils_change_brightness()),
            0x0003 => Ok(Self::scils_signal_aspect_status()),
            0x0004 => Ok(Self::scils_brightness_status()),
            _ => Self::try_as_sci_message_type_from(value),
        }
    }

    #[cfg(feature = "scitds")]
    pub fn try_as_scitds_message_type(&self) -> Result<&str, SciError> {
        match self.0 {
            0x0001 => Ok("FC"),
            0x0002 => Ok("UpdateFillingLevel"),
            0x0003 => Ok("DRFC"),
            0x0008 => Ok("Cancel"),
            0x0006 => Ok("CommandRejected"),
            0x0007 => Ok("TvpsOccupancyStatus"),
            0x0010 => Ok("TvpsFcPFailed"),
            0x0011 => Ok("TvpsFcPAFailed"),
            0x0012 => Ok("AdditionalInformation"),
            0x000B => Ok("TdpStatus"),
            _ => self.try_as_sci_message_type(),
        }
    }

    #[cfg(feature = "scitds")]
    pub fn try_as_scitds_message_type_from(value: u16) -> Result<Self, SciError> {
        match value {
            0x0001 => Ok(Self::scitds_fc()),
            0x0002 => Ok(Self::scitds_update_filling_level()),
            0x0003 => Ok(Self::scitds_drfc()),
            0x0008 => Ok(Self::scitds_cancel()),
            0x0006 => Ok(Self::scitds_command_rejected()),
            0x0007 => Ok(Self::scitds_tvps_occupancy_status()),
            0x0010 => Ok(Self::scitds_tvps_fc_p_failed()),
            0x0011 => Ok(Self::scitds_tvps_fc_p_a_failed()),
            0x0012 => Ok(Self::scitds_additional_information()),
            0x000B => Ok(Self::scitds_tdp_status()),
            _ => Self::try_as_sci_message_type_from(value),
        }
    }
}

impl From<SCIMessageType> for u16 {
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

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SCICloseReason {
    ProtocolError = 1,
    FormalTelegramError = 2,
    ContentTelegramError = 3,
    NormalClose = 4,
    OtherVersionRequired = 5,
    Timeout = 6,
    ChecksumMismatch = 7,
}

impl TryFrom<u8> for SCICloseReason {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::ProtocolError),
            2 => Ok(Self::FormalTelegramError),
            3 => Ok(Self::ContentTelegramError),
            4 => Ok(Self::NormalClose),
            5 => Ok(Self::OtherVersionRequired),
            6 => Ok(Self::Timeout),
            7 => Ok(Self::ChecksumMismatch),
            v => Err(SciError::UnknownCloseReason(v)),
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
    pub fn from_slice(data: &[u8]) -> Self {
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

/// Automatically implement the associated functions for messages
/// with no payload.
#[macro_export]
macro_rules! impl_sci_messages_without_payload {
    ($protocol_type:expr, ($(($message:ident, $message_type:expr)),*)) => {
        impl SCITelegram {
            $(
                pub fn $message(sender: &str, receiver: &str) -> Self {
                    Self {
                        protocol_type: $protocol_type,
                        message_type: $message_type,
                        sender: sender.to_string(),
                        receiver: receiver.to_string(),
                        payload: SCIPayload::default(),
                    }
                }
            )*
        }
    };
}

impl SCITelegram {
    pub fn version_check(
        protocol_type: ProtocolType,
        sender: &str,
        receiver: &str,
        version: u8,
    ) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::pdi_version_check(),
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
            message_type: SCIMessageType::pdi_version_response(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&payload_data),
        }
    }

    pub fn initialisation_request(
        protocol_type: ProtocolType,
        sender: &str,
        receiver: &str,
    ) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::pdi_initialisation_request(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }

    pub fn initialisation_response(
        protocol_type: ProtocolType,
        sender: &str,
        receiver: &str,
    ) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::pdi_initialisation_response(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }

    pub fn initialisation_completed(
        protocol_type: ProtocolType,
        sender: &str,
        receiver: &str,
    ) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::pdi_initialisation_completed(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }

    pub fn close(
        protocol_type: ProtocolType,
        sender: &str,
        receiver: &str,
        close_reason: SCICloseReason,
    ) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::pdi_close(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[close_reason as u8]),
        }
    }

    pub fn release_for_maintenance(
        protocol_type: ProtocolType,
        sender: &str,
        receiver: &str,
    ) -> Self {
        Self {
            protocol_type,
            message_type: SCIMessageType::pdi_release_for_maintenance(),
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
        let message_type_as_u16 = u16::from_le_bytes(value[1..3].try_into().unwrap());
        let message_type = match protocol_type {
            #[cfg(feature = "scip")]
            ProtocolType::SCIProtocolP => {
                SCIMessageType::try_as_scip_message_type_from(message_type_as_u16)?
            }
            #[cfg(feature = "scils")]
            ProtocolType::SCIProtocolLS => {
                SCIMessageType::try_as_scils_message_type_from(message_type_as_u16)?
            }
            #[cfg(feature = "scitds")]
            ProtocolType::SCIProtocolTDS => {
                SCIMessageType::try_as_scitds_message_type_from(message_type_as_u16)?
            }
            _ => unimplemented!(),
        };
        Ok(Self {
            protocol_type,
            message_type,
            sender: String::from_utf8_lossy(&value[3..23]).to_string(),
            receiver: String::from_utf8_lossy(&value[23..43]).to_string(),
            payload: SCIPayload::from_slice(&value[43..]),
        })
    }
}

impl From<SCITelegram> for Vec<u8> {
    fn from(val: SCITelegram) -> Self {
        let mut data = vec![val.protocol_type as u8];
        let message_type: u16 = val.message_type.into();
        data.append(&mut message_type.to_le_bytes().to_vec());
        data.append(&mut str_to_sci_name(&val.sender));
        data.append(&mut str_to_sci_name(&val.receiver));
        if val.payload.used > 0 {
            let mut payload = Vec::from(val.payload.data);
            data.append(&mut payload);
        }
        data
    }
}

/// The SCI equivalent of [`rasta_rs::RastaCommand`].
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
