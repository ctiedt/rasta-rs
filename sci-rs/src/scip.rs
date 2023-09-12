//! # SCI Point
//!
//! The Standard Communication Interface for points.

#[derive(Debug, Clone, Copy)]
pub enum SciPError {
    UnknownTargetLocation(u8),
    UnknownLocation(u8),
}

use crate::{impl_sci_message_type, SciError};

use super::{ProtocolType, SCIMessageType, SCIPayload, SCITelegram};

impl_sci_message_type!(
    (scip_change_location, 0x0001),
    (scip_location_status, 0x000B)
);

/// The target location of [`SCITelegram::change_location`].
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum SCIPointTargetLocation {
    PointLocationChangeToRight = 0x01,
    PointLocationChangeToLeft = 0x02,
}

impl TryFrom<u8> for SCIPointTargetLocation {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::PointLocationChangeToRight),
            0x02 => Ok(Self::PointLocationChangeToLeft),
            v => Err(SciPError::UnknownTargetLocation(v).into()),
        }
    }
}

/// The current location of a point. This is different from
/// [`SCIPointTargetLocation`] in that it supports locations
/// that cannot be manually requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SCIPointLocation {
    PointLocationRight = 0x01,
    PointLocationLeft = 0x02,
    PointNoTargetLocation = 0x03,
    PointBumped = 0x04,
}

impl TryFrom<u8> for SCIPointLocation {
    type Error = SciError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::PointLocationRight),
            0x02 => Ok(Self::PointLocationLeft),
            0x03 => Ok(Self::PointNoTargetLocation),
            0x04 => Ok(Self::PointBumped),
            v => Err(SciPError::UnknownLocation(v).into()),
        }
    }
}

impl SCITelegram {
    pub fn change_location(sender: &str, receiver: &str, to: SCIPointTargetLocation) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolP,
            message_type: SCIMessageType::scip_change_location(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[to as u8]),
        }
    }

    pub fn location_status(sender: &str, receiver: &str, location: SCIPointLocation) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolP,
            message_type: SCIMessageType::scip_location_status(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[location as u8]),
        }
    }
}
