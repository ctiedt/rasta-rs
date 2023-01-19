use std::collections::HashMap;

use crate::{
    message::RastaId, RastaConnection, RastaConnectionState, RastaError, RastaListener,
    RASTA_TIMEOUT_DURATION,
};

use super::{ProtocolType, SCICommand, SCIMessageType, SCIPayload, SCITelegram};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum SCIPointTargetLocation {
    PointLocationChangeToRight = 0x01,
    PointLocationChangeToLeft = 0x02,
}

impl TryFrom<u8> for SCIPointTargetLocation {
    type Error = RastaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::PointLocationChangeToRight),
            0x02 => Ok(Self::PointLocationChangeToLeft),
            v => Err(RastaError::Other(format!(
                "Unknown SCIP target location: {v}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SCIPointLocation {
    PointLocationRight = 0x01,
    PointLocationLeft = 0x02,
    PointNoTargetLocation = 0x03,
    PointBumped = 0x04,
}

impl TryFrom<u8> for SCIPointLocation {
    type Error = RastaError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::PointLocationRight),
            0x02 => Ok(Self::PointLocationLeft),
            0x03 => Ok(Self::PointNoTargetLocation),
            0x04 => Ok(Self::PointBumped),
            v => Err(RastaError::Other(format!("Unknown SCIP location: {v}"))),
        }
    }
}

impl SCITelegram {
    pub fn change_location(sender: &str, receiver: &str, to: SCIPointTargetLocation) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolP,
            message_type: SCIMessageType::SCIPMessageTypeChangeLocation,
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[to as u8]),
        }
    }

    pub fn location_status(sender: &str, receiver: &str, location: SCIPointLocation) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolP,
            message_type: SCIMessageType::SCIPMessageTypeLocationStatus,
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::from_slice(&[location as u8]),
        }
    }

    pub fn timeout(sender: &str, receiver: &str) -> Self {
        Self {
            protocol_type: ProtocolType::SCIProtocolP,
            message_type: SCIMessageType::SCIPMessageTypeTimeout,
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            payload: SCIPayload::default(),
        }
    }
}

pub struct SCIPListener {
    listener: RastaListener,
    name: String,
}

impl SCIPListener {
    pub fn new(listener: RastaListener, name: String) -> Self {
        Self { listener, name }
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

pub struct SCIPConnection {
    conn: RastaConnection,
    name: String,
    sci_name_rasta_id_mapping: HashMap<String, RastaId>,
}
impl SCIPConnection {
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

    pub fn run<F>(&mut self, mut telegram_fn: F) -> Result<(), RastaError>
    where
        F: FnMut(Option<SCITelegram>) -> SCICommand,
    {
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
