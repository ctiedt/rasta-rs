//! # SCI Point
//!
//! The Standard Communication Interface for points.

#[derive(Debug, Clone, Copy)]
pub enum SciPError {
    UnknownTargetLocation(u8),
    UnknownLocation(u8),
}

impl Display for SciPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SciPError {}

use std::fmt::Display;

use crate::impl_sci_message_type;

use super::{ProtocolType, SCIMessageType, SCIPayload, SCITelegram};

impl_sci_message_type!(
    (scip_change_location, 0x0001),
    (scip_location_status, 0x000B)
);

enumerate! {
    SCIPointTargetLocation,
    "The target location of [`SCITelegram::change_location`].",
    u8,
    SciPError::UnknownTargetLocation, {
    PointLocationChangeToRight = 0x01,
    PointLocationChangeToLeft = 0x02
}}

enumerate! {
    SCIPointLocation,
    "The current location of a point. This is different from [`SCIPointTargetLocation`] in that it supports locations that cannot be manually requested.",
    u8,
    SciPError::UnknownLocation,
    {
        PointLocationRight = 0x01,
    PointLocationLeft = 0x02,
    PointNoTargetLocation = 0x03,
    PointBumped = 0x04
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
