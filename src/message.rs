use std::ops::Deref;

use crate::RastaError;

pub type RastaId = u32;

pub const RASTA_VERSION: [u8; 4] = [0x30, 0x33, 0x30, 0x31];

pub struct Message {
    pub content: Vec<u8>,
    data_len: Option<usize>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            content: vec![0; 1024],
            data_len: None,
        }
    }
}

#[derive(Default)]
pub struct MessageBuilder {
    msg: Message,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            msg: Message::default(),
        }
    }

    pub fn length(mut self, len: u16) -> Self {
        self.msg.content[0..2].copy_from_slice(&len.to_ne_bytes());
        self
    }

    pub fn message_type(mut self, message_type: MessageType) -> Self {
        self.msg.content[3..5].copy_from_slice(&(message_type as u16).to_ne_bytes());
        self
    }

    pub fn receiver(mut self, receiver: RastaId) -> Self {
        self.msg.content[6..10].copy_from_slice(&receiver.to_ne_bytes());
        self
    }

    pub fn sender(mut self, sender: RastaId) -> Self {
        self.msg.content[10..14].copy_from_slice(&sender.to_ne_bytes());
        self
    }

    pub fn sequence_number(mut self, sequence_number: u32) -> Self {
        self.msg.content[15..19].copy_from_slice(&sequence_number.to_ne_bytes());
        self
    }

    pub fn confirmed_sequence_number(mut self, confirmed_sequence_number: u32) -> Self {
        self.msg.content[19..23].copy_from_slice(&confirmed_sequence_number.to_ne_bytes());
        self
    }

    pub fn timestamp(mut self, timestamp: u32) -> Self {
        self.msg.content[24..28].copy_from_slice(&timestamp.to_ne_bytes());
        self
    }

    pub fn confirmed_timestamp(mut self, confirmed_timestamp: u32) -> Self {
        self.msg.content[29..33].copy_from_slice(&confirmed_timestamp.to_ne_bytes());
        self
    }

    pub fn data(mut self, data: &[u8]) -> Self {
        self.msg.content[34..(34 + data.len())].copy_from_slice(data);
        self.msg.data_len.replace(data.len());
        self
    }

    pub fn security_code(mut self, code: &[u8; 8]) -> Self {
        let len = self.msg.content.len();
        self.msg.content[(len - 8)..len].copy_from_slice(code);
        self
    }

    pub fn build(self) -> Message {
        self.msg
    }
}

impl Message {
    pub fn length(&self) -> u16 {
        u16::from_ne_bytes(self.content[0..2].try_into().unwrap())
    }

    pub fn message_type(&self) -> MessageType {
        let msg_type = u16::from_ne_bytes(self.content[3..5].try_into().unwrap());
        MessageType::try_from(msg_type).unwrap()
    }

    pub fn receiver(&self) -> RastaId {
        u32::from_ne_bytes(self.content[6..10].try_into().unwrap())
    }

    pub fn sender(&self) -> RastaId {
        u32::from_ne_bytes(self.content[10..14].try_into().unwrap())
    }

    pub fn sequence_number(&self) -> u32 {
        u32::from_ne_bytes(self.content[15..19].try_into().unwrap())
    }

    pub fn confirmed_sequence_number(&self) -> u32 {
        u32::from_ne_bytes(self.content[19..23].try_into().unwrap())
    }

    pub fn timestamp(&self) -> u32 {
        u32::from_ne_bytes(self.content[24..28].try_into().unwrap())
    }

    pub fn confirmed_timestamp(&self) -> u32 {
        u32::from_ne_bytes(self.content[29..33].try_into().unwrap())
    }

    pub fn data(&self) -> &[u8] {
        &self.content[34..(34 + self.data_len.unwrap())]
    }

    pub fn security_code(&self) -> &[u8] {
        let len = self.content.len();
        &self.content[(len - 8)..len]
    }

    pub fn connection_request(
        receiver: RastaId,
        sender: RastaId,
        timestamp: u32,
        n_sendmax: u16,
    ) -> Self {
        let mut data = [0; 14];
        data[..4].copy_from_slice(&RASTA_VERSION);
        data[5..7].copy_from_slice(&n_sendmax.to_ne_bytes());
        let initial_seq_nr = if cfg!(feature = "rand") {
            rand::random()
        } else {
            4
        };
        MessageBuilder::new()
            .length(50)
            .message_type(MessageType::ConnReq)
            .receiver(receiver)
            .sender(sender)
            .sequence_number(initial_seq_nr)
            .confirmed_sequence_number(0)
            .timestamp(timestamp)
            .confirmed_timestamp(0)
            .data(&data)
            .security_code(&[0; 8])
            .build()
    }

    pub fn connection_response(
        receiver: RastaId,
        sender: RastaId,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
        n_sendmax: u16,
    ) -> Self {
        let mut data = [0; 14];
        data[..4].copy_from_slice(&RASTA_VERSION);
        data[5..7].copy_from_slice(&n_sendmax.to_ne_bytes());
        let sequence_number = confirmed_sequence_number + 1;
        MessageBuilder::new()
            .length(50)
            .message_type(MessageType::ConnResp)
            .receiver(receiver)
            .sender(sender)
            .sequence_number(sequence_number)
            .confirmed_sequence_number(confirmed_sequence_number)
            .timestamp(timestamp)
            .confirmed_timestamp(confirmed_timestamp)
            .data(&data)
            .security_code(&[0; 8])
            .build()
    }

    pub fn retransmission_request(
        receiver: RastaId,
        sender: RastaId,
        sequence_number: u32,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
    ) -> Self {
        MessageBuilder::new()
            .length(36)
            .message_type(MessageType::RetrReq)
            .receiver(receiver)
            .sender(sender)
            .sequence_number(sequence_number)
            .confirmed_sequence_number(confirmed_sequence_number)
            .timestamp(timestamp)
            .confirmed_timestamp(confirmed_timestamp)
            .data(&[])
            .security_code(&[0; 8])
            .build()
    }

    pub fn retransmission_response(
        receiver: RastaId,
        sender: RastaId,
        sequence_number: u32,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
    ) -> Self {
        MessageBuilder::new()
            .length(36)
            .message_type(MessageType::RetrResp)
            .receiver(receiver)
            .sender(sender)
            .sequence_number(sequence_number)
            .confirmed_sequence_number(confirmed_sequence_number)
            .timestamp(timestamp)
            .confirmed_timestamp(confirmed_timestamp)
            .data(&[])
            .security_code(&[0; 8])
            .build()
    }

    pub fn heartbeat(
        receiver: RastaId,
        sender: RastaId,
        sequence_number: u32,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
    ) -> Self {
        MessageBuilder::new()
            .length(36)
            .message_type(MessageType::HB)
            .receiver(receiver)
            .sender(sender)
            .sequence_number(sequence_number)
            .confirmed_sequence_number(confirmed_sequence_number)
            .timestamp(timestamp)
            .confirmed_timestamp(confirmed_timestamp)
            .data(&[])
            .security_code(&[0; 8])
            .build()
    }

    pub fn disconnection_request(
        receiver: RastaId,
        sender: RastaId,
        sequence_number: u32,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
    ) -> Self {
        MessageBuilder::new()
            .length(40)
            .message_type(MessageType::DiscReq)
            .receiver(receiver)
            .sender(sender)
            .sequence_number(sequence_number)
            .confirmed_sequence_number(confirmed_sequence_number)
            .timestamp(timestamp)
            .confirmed_timestamp(confirmed_timestamp)
            .data(&[])
            .security_code(&[0; 8])
            .build()
    }

    pub fn data_message(
        receiver: RastaId,
        sender: RastaId,
        sequence_number: u32,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
        data: &[u8],
    ) -> Self {
        MessageBuilder::new()
            .length((36 + data.len()) as u16)
            .message_type(MessageType::Data)
            .receiver(receiver)
            .sender(sender)
            .sequence_number(sequence_number)
            .confirmed_sequence_number(confirmed_sequence_number)
            .timestamp(timestamp)
            .confirmed_timestamp(confirmed_timestamp)
            .data(data)
            .security_code(&[0; 8])
            .build()
    }

    pub fn retransmitted_data_message(
        receiver: RastaId,
        sender: RastaId,
        sequence_number: u32,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
        data: &[u8],
    ) -> Self {
        MessageBuilder::new()
            .length((36 + data.len()) as u16)
            .message_type(MessageType::RetrData)
            .receiver(receiver)
            .sender(sender)
            .sequence_number(sequence_number)
            .confirmed_sequence_number(confirmed_sequence_number)
            .timestamp(timestamp)
            .confirmed_timestamp(confirmed_timestamp)
            .data(data)
            .security_code(&[0; 8])
            .build()
    }
}

impl From<&[u8]> for Message {
    fn from(val: &[u8]) -> Self {
        let mut content = Vec::new();
        content.extend_from_slice(val);
        let length = u16::from_ne_bytes(content[0..2].try_into().unwrap());
        let data_len = length - 36;
        Self {
            content,
            data_len: Some(data_len.into()),
        }
    }
}

impl Deref for Message {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

#[derive(PartialEq, Eq, Debug)]
#[repr(u16)]
pub enum MessageType {
    ConnReq = 6200,
    ConnResp = 6201,
    RetrReq = 6212,
    RetrResp = 6213,
    DiscReq = 6216,
    HB = 6220,
    Data = 6240,
    RetrData = 6241,
}

impl TryFrom<u16> for MessageType {
    type Error = RastaError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            6200 => Ok(Self::ConnReq),
            6201 => Ok(Self::ConnResp),
            6212 => Ok(Self::RetrReq),
            6213 => Ok(Self::RetrResp),
            6216 => Ok(Self::DiscReq),
            6220 => Ok(Self::HB),
            6240 => Ok(Self::Data),
            6241 => Ok(Self::RetrData),
            n => Err(RastaError::Other(format!(
                "Value {n} is not a valid Message Type"
            ))),
        }
    }
}
