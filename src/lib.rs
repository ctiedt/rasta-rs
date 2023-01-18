//#![no_std]

use message::{Message, MessageType, RastaId};

pub mod message;

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

const N_SENDMAX: u16 = u16::MAX;

#[derive(Debug)]
pub enum RastaError {
    InvalidSeqNr,
    Timeout,
    Other(String),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RastaConnectionState {
    Closed,
    Down,
    Start,
    Up,
}
pub enum RastaCommand<D: AsRef<[u8]>> {
    Data(D),
    Wait,
    Disconnect,
}

pub struct RastaListener {
    listener: TcpListener,
    connections: Vec<RastaId>,
    on_receive: fn(Message),
}

impl RastaListener {
    pub fn new<S: ToSocketAddrs>(addr: S, on_receive: fn(Message)) -> Self {
        let listener = TcpListener::bind(addr).unwrap();
        Self {
            listener,
            connections: Vec::new(),
            on_receive,
        }
    }

    fn timestamp(&self) -> u32 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
    }

    pub fn listen(&mut self) {
        for conn in self.listener.incoming() {
            let mut conn = conn.unwrap();
            println!("New connection: {}", conn.peer_addr().unwrap());
            loop {
                let mut buf = vec![0; 1024];
                conn.read(&mut buf).unwrap();
                let msg = Message::from(buf.as_slice());
                dbg!(msg.message_type());
                match msg.message_type() {
                    MessageType::ConnReq => {
                        let resp = Message::connection_response(
                            msg.sender(),
                            msg.receiver(),
                            msg.sequence_number(),
                            self.timestamp(),
                            msg.timestamp(),
                            N_SENDMAX,
                        );
                        conn.write(&resp).unwrap();
                        self.connections.push(msg.sender());
                    }
                    MessageType::ConnResp => {
                        //Ignore
                    }
                    MessageType::RetrReq => unimplemented!(),
                    MessageType::RetrResp => unimplemented!(),
                    MessageType::DiscReq => {
                        if let Some(idx) = self.connections.iter().position(|c| *c == msg.sender())
                        {
                            self.connections.remove(idx);
                            break;
                        }
                    }
                    MessageType::HB => {
                        if self.connections.contains(&msg.sender()) {
                            println!("Heartbeat from {}", msg.sender());
                            let response = Message::heartbeat(
                                msg.sender(),
                                msg.receiver(),
                                msg.sequence_number() + 1,
                                msg.sequence_number(),
                                self.timestamp(),
                                msg.timestamp(),
                            );
                            conn.write(&response).unwrap();
                        }
                    }
                    MessageType::Data => {
                        if self.connections.contains(&msg.sender()) {
                            println!("Received data from {}", msg.sender());
                            (self.on_receive)(msg);
                        }
                    }
                    MessageType::RetrData => unimplemented!("Handled by TCP"),
                }
            }
        }
    }
}

pub struct RastaConnection {
    state: RastaConnectionState,
    id: RastaId,
    peer: RastaId,
    seq_nr: u32,
    confirmed_timestamp: u32,
    server: TcpStream,
}

impl RastaConnection {
    pub fn new<S: ToSocketAddrs>(server: S, id: RastaId) -> Self {
        let connection = TcpStream::connect(server).unwrap();
        Self {
            state: RastaConnectionState::Closed,
            id,
            peer: 0,
            seq_nr: 0,
            confirmed_timestamp: 0,
            server: connection,
        }
    }

    fn next_seq_nr(&mut self) -> (u32, u32) {
        self.seq_nr = self.seq_nr + 1;
        (self.seq_nr - 1, self.seq_nr)
    }

    fn timestamp(&self) -> u32 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
    }

    pub fn open_connection(&mut self, receiver: u32) -> Result<(), RastaError> {
        let msg = Message::connection_request(receiver, self.id, self.timestamp(), N_SENDMAX);
        self.server.write(&msg).unwrap();
        let mut buf = vec![0; 1024];
        self.server.read(&mut buf).unwrap();
        let response = Message::from(buf.as_slice());
        if response.message_type() == MessageType::ConnResp {
            self.state = RastaConnectionState::Up;
            self.seq_nr = response.sequence_number();
            self.confirmed_timestamp = response.timestamp();
            self.peer = response.sender();
            println!("Connected to {}", self.server.peer_addr().unwrap());
        }
        Ok(())
    }

    pub fn close_connection(&mut self) -> Result<(), RastaError> {
        if self.connection_state_request() != RastaConnectionState::Up {
            Ok(())
        } else {
            let (confirmed_seq_nr, seq_nr) = self.next_seq_nr();
            let msg = Message::disconnection_request(
                self.peer,
                self.id,
                seq_nr,
                confirmed_seq_nr,
                self.timestamp(),
                self.confirmed_timestamp,
            );
            self.server.write(&msg).unwrap();
            self.state = RastaConnectionState::Closed;
            Ok(())
        }
    }

    pub fn send_data(&mut self, data: &[u8]) -> Result<(), RastaError> {
        let (confirmed_seq_nr, seq_nr) = self.next_seq_nr();
        let msg = Message::data_message(
            self.peer,
            self.id,
            seq_nr,
            confirmed_seq_nr,
            self.timestamp(),
            self.confirmed_timestamp,
            data,
        );
        self.server.write(&msg).unwrap();
        Ok(())
    }

    pub fn send_heartbeat(&mut self) -> Result<(), RastaError> {
        let (confirmed_seq_nr, seq_nr) = self.next_seq_nr();
        let msg = Message::heartbeat(
            self.peer,
            self.id,
            seq_nr,
            confirmed_seq_nr,
            self.timestamp(),
            self.confirmed_timestamp,
        );
        self.server.write(&msg).unwrap();
        let mut buf = vec![0; 1024];
        self.server.read(&mut buf).unwrap();
        let response = Message::from(buf.as_slice());
        if response.message_type() == MessageType::HB {
            self.seq_nr = response.sequence_number();
            self.confirmed_timestamp = response.timestamp();
        }
        Ok(())
    }

    pub fn connection_state_request(&self) -> RastaConnectionState {
        self.state
    }

    pub fn run<F, D>(&mut self, peer: RastaId, mut message_fn: F)
    where
        F: FnMut() -> RastaCommand<D>,
        D: AsRef<[u8]>,
    {
        self.open_connection(peer).unwrap();
        loop {
            match message_fn() {
                RastaCommand::Data(data) => {
                    self.send_data(data.as_ref()).unwrap();
                }
                RastaCommand::Wait => self.send_heartbeat().unwrap(),
                RastaCommand::Disconnect => {
                    self.close_connection().unwrap();
                    break;
                }
            }
        }
    }
}

impl Drop for RastaConnection {
    fn drop(&mut self) {
        self.close_connection().unwrap();
    }
}

mod tests {
    #[test]
    fn test_conn_req_len() {}
}
