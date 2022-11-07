//#![no_std]

use message::Message;

pub mod message;
mod udp;

pub enum RastaError {
    Other(String),
}

pub struct RastaConnection {}

impl RastaConnection {
    pub fn open_connection(sender: u32, receiver: u32, network: u32) -> Result<Self, RastaError> {
        todo!()
    }

    pub fn close_connection(&mut self, receiver: u32, network: u32) -> Result<(), RastaError> {
        todo!()
    }

    pub fn send_data<const N: usize>(
        &mut self,
        receiver: u32,
        network: u32,
        data: Message<N>,
    ) -> Result<(), RastaError> {
        todo!()
    }

    pub fn receive_data<const N: usize>(
        &mut self,
        receiver: u32,
        network: u32,
    ) -> Result<Message<N>, RastaError> {
        todo!()
    }

    pub fn connection_state_request(&self) -> Result<(), RastaError> {
        todo!()
    }
}

mod tests {
    #[test]
    fn test_conn_req_len() {}
}
