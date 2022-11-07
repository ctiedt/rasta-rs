use std::ops::Deref;

pub struct Message<const N: usize> {
    content: [u8; N],
    data_len: Option<usize>,
}

impl<const N: usize> Default for Message<N> {
    fn default() -> Self {
        Self {
            content: [0; N],
            data_len: None,
        }
    }
}

impl<const N: usize> Message<N> {
    pub fn length(mut self, len: u16) -> Self {
        self.content[0..2].copy_from_slice(&len.to_ne_bytes());
        self
    }

    pub fn message_type(mut self, message_type: MessageType) -> Self {
        self.content[3..5].copy_from_slice(&(message_type as u16).to_ne_bytes());
        self
    }

    pub fn receiver(mut self, receiver: u32) -> Self {
        self.content[6..10].copy_from_slice(&receiver.to_ne_bytes());
        self
    }

    pub fn sender(mut self, sender: u32) -> Self {
        self.content[10..14].copy_from_slice(&sender.to_ne_bytes());
        self
    }

    pub fn sequence_number(mut self, sequence_number: u32) -> Self {
        self.content[15..19].copy_from_slice(&sequence_number.to_ne_bytes());
        self
    }

    pub fn confirmed_sequence_number(mut self, confirmed_sequence_number: u32) -> Self {
        self.content[19..23].copy_from_slice(&confirmed_sequence_number.to_ne_bytes());
        self
    }

    pub fn timestamp(mut self, timestamp: u32) -> Self {
        self.content[24..28].copy_from_slice(&timestamp.to_ne_bytes());
        self
    }

    pub fn confirmed_timestamp(mut self, confirmed_timestamp: u32) -> Self {
        self.content[29..33].copy_from_slice(&confirmed_timestamp.to_ne_bytes());
        self
    }

    pub fn data(mut self, data: &[u8]) -> Self {
        self.content[34..(34 + data.len())].copy_from_slice(data);
        self.data_len.replace(data.len());
        self
    }

    pub fn security_code(mut self, code: &[u8; 8]) -> Self {
        self.content[(N - 8)..N].copy_from_slice(code);
        self
    }
}

impl Message<50> {
    pub fn connection_request(receiver: u32, sender: u32, timestamp: u32, n_sendmax: u16) -> Self {
        let mut data = [0; 14];
        data[..4].copy_from_slice(&[0x30, 0x33, 0x30, 0x31]);
        dbg!(&n_sendmax.to_ne_bytes());
        data[5..7].copy_from_slice(&n_sendmax.to_ne_bytes());
        Self::default()
            .length(50)
            .message_type(MessageType::ConnReq)
            .receiver(receiver)
            .sender(sender)
            .sequence_number(4)
            .confirmed_sequence_number(0)
            .timestamp(timestamp)
            .confirmed_timestamp(0)
            .data(&data)
            .security_code(&[0; 8])
    }

    pub fn connection_response(
        receiver: u32,
        sender: u32,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
        n_sendmax: u16,
    ) -> Self {
        let mut data = [0; 14];
        data[..4].copy_from_slice(&[0x30, 0x33, 0x30, 0x31]);
        data[5..7].copy_from_slice(&n_sendmax.to_ne_bytes());
        Self::default()
            .length(50)
            .message_type(MessageType::ConnResp)
            .receiver(receiver)
            .sender(sender)
            .sequence_number(4)
            .confirmed_sequence_number(confirmed_sequence_number)
            .timestamp(timestamp)
            .confirmed_timestamp(confirmed_timestamp)
            .data(&data)
            .security_code(&[0; 8])
    }
}

impl Message<36> {
    pub fn retransmission_request(
        receiver: u32,
        sender: u32,
        sequence_number: u32,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
    ) -> Self {
        Self::default()
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
    }

    pub fn retransmission_response(
        receiver: u32,
        sender: u32,
        sequence_number: u32,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
    ) -> Self {
        Self::default()
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
    }

    pub fn heartbeat(
        receiver: u32,
        sender: u32,
        sequence_number: u32,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
    ) -> Self {
        Self::default()
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
    }
}

impl Message<40> {
    pub fn disconnection_request(
        receiver: u32,
        sender: u32,
        sequence_number: u32,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
    ) -> Self {
        Self::default()
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
    }
}

impl<const N: usize> Message<N> {
    pub fn data_message(
        receiver: u32,
        sender: u32,
        sequence_number: u32,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
        data: &[u8],
    ) -> Self {
        Self::default()
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
    }

    pub fn retransmitted_data_message(
        receiver: u32,
        sender: u32,
        sequence_number: u32,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
        data: &[u8],
    ) -> Self {
        Self::default()
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
    }
}

impl<const N: usize> From<Message<N>> for [u8; N] {
    fn from(m: Message<N>) -> Self {
        m.content
    }
}

impl<const N: usize> Deref for Message<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

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
