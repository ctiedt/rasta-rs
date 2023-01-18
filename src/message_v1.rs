#[repr(packed)]
pub struct Message<const N: usize> {
    length: u16,
    message_type: MessageType,
    receiver: u32,
    sender: u32,
    sequence_number: u32,
    confirmed_sequence_number: u32,
    timestamp: u32,
    confirmed_timestamp: u32,
    pub data: [u8; N],
    security_code: [u8; 8],
}

impl Message<14> {
    pub fn connection_request(receiver: u32, sender: u32, timestamp: u32, n_sendmax: u16) -> Self {
        let sendmax = n_sendmax.to_ne_bytes();
        Self {
            length: 50,
            message_type: MessageType::ConnReq,
            receiver,
            sender,
            // Todo: Replace with random number
            sequence_number: 4,
            confirmed_sequence_number: 0,
            timestamp,
            confirmed_timestamp: 0,
            data: [
                0x30, 0x33, 0x30, 0x31, sendmax[0], sendmax[1], 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            security_code: [0; 8],
        }
    }

    pub fn connection_response(
        receiver: u32,
        sender: u32,
        confirmed_sequence_number: u32,
        timestamp: u32,
        confirmed_timestamp: u32,
        n_sendmax: u16,
    ) -> Self {
        let sendmax = n_sendmax.to_ne_bytes();
        Self {
            length: 50,
            message_type: MessageType::ConnReq,
            receiver,
            sender,
            // Todo: Replace with random number
            sequence_number: 4,
            confirmed_sequence_number,
            timestamp,
            confirmed_timestamp,
            data: [
                0x30, 0x33, 0x30, 0x31, sendmax[0], sendmax[1], 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            security_code: [0; 8],
        }
    }
}

impl<const N: usize> From<Message<N>> for [u8; N + 36] {
    fn from(m: Message<N>) -> Self {
        unsafe { core::mem::transmute::<Message<N>, [u8; N + 36]>(m) }
    }
}

#[repr(u16)]
pub enum MessageType {
    ConnReq = 6200,
    ConnResp = 6201,
    RetrReq = 6212,
    RetrResp = 6213,
    HB = 6220,
    Data = 6240,
    RetrData = 6241,
}
