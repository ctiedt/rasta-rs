//! # rasta-rs
//!
//! A simplified implementation of the Rail Safe Transport Application Protocol (RaSTA) in Rust.
//! This implementation only provides very basic functionality, no redundancy and no
//! explicit retransmission (since it is TCP-based).
//!
//! ## Example - Sending:
//!
//! ```rust
//! let addr: SocketAddrV4 = "127.0.0.1:8888".parse()?;
//! // Connect to receiver on localhost
//! // using RaSTA ID 1234 for sender
//! let mut conn = RastaConnection::try_new(addr, 1234)?;
//! let mut sent = false;
//! // Connect to receiver with ID 5678
//! conn.run(5678, |data| {
//!     // Data is Some() if a new message has been receiver
//!     if !sent {
//!         sent = true;
//!         RastaCommand::Data(vec![1, 2, 3, 4])
//!     } else {
//!         if let Some(data) = data {
//!             dbg!(data);
//!         }
//!         // RastaCommand controls the flow of messages
//!         RastaCommand::Wait
//!     }
//! })?;
//! ```
//!
//! ## Example - Receiving:
//!
//! ```rust
//! let addr: SocketAddrV4 = "127.0.0.1:8888".parse()?;
//! // Listen on localhost with RaSTA ID 5678
//! let mut conn = RastaListener::try_new(addr, 5678)?;
//! conn.listen(|msg| {
//!     dbg!(msg.data());
//!     // Return Some() to respond with data to message
//!     Some(vec![5, 6, 7, 8])
//! })?;
//! ```

use message::{Message, MessageType, RastaId, RASTA_VERSION};

pub mod message;
pub mod sci;

use std::{
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    time::{Duration, Instant},
};

#[cfg(feature = "wasi_sockets")]
use std::os::wasi::io::FromRawFd;

/// The maximum number of messages in a [`RastaConnection`] or [`RastaListener`] buffer.
pub const N_SENDMAX: u16 = u16::MAX;
/// The timeout duration for messages between a [`RastaConnection`] and [`RastaListener`].
pub const RASTA_TIMEOUT_DURATION: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub enum RastaError {
    InvalidSeqNr,
    StateError,
    Timeout,
    VersionMismatch,
    IOError(std::io::Error),
    Other(String),
}

impl From<std::io::Error> for RastaError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::TimedOut => Self::Timeout,
            _ => Self::IOError(value),
        }
    }
}

/// The State of a RaSTA connection as defined in the specification.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RastaConnectionState {
    Closed,
    Down,
    Start,
    Up,
}

/// The control flow in a RaSTA connection.
/// Determines which messages a [`RastaConnection`]
/// should send.
pub enum RastaCommand<D: AsRef<[u8]>> {
    /// Send a data messages constructed from the passed buffer.
    Data(D),
    /// Do not send any data, but maintain the connection. Sends a Heartbeat.
    Wait,
    /// Terminate the connection.
    Disconnect,
}

/// This type roughly corresponds to [`std::net::TcpListener`].
/// Create it using [`RastaListener::try_new`] and then handle
/// messages using [`RastaListener::listen`]. Alternatively, you
/// can manage the connection yourself. If you want to do this,
/// look at the implementation of [`RastaListener::listen`] for
/// inspiration.
pub struct RastaListener {
    listener: TcpListener,
    connections: Vec<RastaId>,
    id: RastaId,
    seq_nr: Option<u32>,
    last_message_timestamp: Option<Instant>,
}

impl RastaListener {
    pub fn try_new<S: ToSocketAddrs>(addr: S, id: RastaId) -> Result<Self, RastaError> {
        #[cfg(feature = "wasi_sockets")]
        let listener = unsafe { TcpListener::from_raw_fd(3) };
        #[cfg(not(feature = "wasi_sockets"))]
        let listener = TcpListener::bind(addr).map_err(RastaError::from)?;
        Ok(Self {
            listener,
            connections: Vec::new(),
            id,
            seq_nr: None,
            last_message_timestamp: None,
        })
    }

    fn timestamp(&self) -> u32 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
    }

    pub fn listen<F, D>(&mut self, mut on_receive: F) -> Result<(), RastaError>
    where
        F: FnMut(Message) -> Option<D>,
        D: AsRef<[u8]>,
    {
        for conn in self.listener.incoming() {
            if let Err(e) = &conn {
                if e.kind() == ErrorKind::WouldBlock {
                    continue;
                }
            }
            let mut conn = conn.map_err(RastaError::from)?;
            #[cfg(not(feature = "wasi_sockets"))]
            conn.set_read_timeout(Some(RASTA_TIMEOUT_DURATION))
                .map_err(RastaError::from)?;
            #[cfg(not(feature = "wasi_sockets"))]
            println!(
                "New connection: {}",
                conn.peer_addr().map_err(RastaError::from)?
            );
            #[cfg(feature = "wasi_sockets")]
            println!("New connection!");
            loop {
                let mut buf = vec![0; 1024];
                let conn_result = conn.read(&mut buf);
                if conn_result.is_err() {
                    let c = self.connections.pop();
                    println!("Client {} unexpectedly disconnected", c.unwrap());
                    self.seq_nr = None;
                    break;
                } else if conn_result.as_ref().unwrap() == &0 {
                    println!("Invalid message received - aborting connection");
                    self.seq_nr = None;
                    break;
                }
                let msg = Message::from(&buf[..conn_result.unwrap()]);
                dbg!(msg.message_type());
                dbg!(msg.sender());
                dbg!(msg.receiver());
                dbg!(msg.sequence_number());
                dbg!(msg.confirmed_sequence_number());
                dbg!(self.seq_nr);
                if self.seq_nr.is_some() && msg.confirmed_sequence_number() != self.seq_nr.unwrap()
                {
                    dbg!(msg.confirmed_sequence_number(), self.seq_nr.unwrap());
                    return Err(RastaError::InvalidSeqNr);
                }
                if self.last_message_timestamp.is_some()
                    && Instant::now().duration_since(self.last_message_timestamp.unwrap())
                        > RASTA_TIMEOUT_DURATION
                {
                    let response = Message::disconnection_request(
                        msg.sender(),
                        msg.receiver(),
                        msg.sequence_number() + 1,
                        msg.sequence_number(),
                        self.timestamp(),
                        msg.timestamp(),
                    );
                    conn.write(&response).map_err(RastaError::from)?;
                    break;
                }
                self.seq_nr.replace(msg.sequence_number());
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
                        conn.write(&resp).map_err(RastaError::from)?;
                        self.seq_nr.replace(msg.sequence_number() + 1);
                        self.connections.push(msg.sender());
                    }
                    MessageType::ConnResp => {
                        //Ignore
                    }
                    MessageType::RetrReq => unimplemented!("Handled by TCP"),
                    MessageType::RetrResp => unimplemented!("Handled by TCP"),
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
                            self.seq_nr.replace(msg.sequence_number() + 1);
                            let response = Message::heartbeat(
                                msg.sender(),
                                msg.receiver(),
                                self.seq_nr.unwrap(),
                                msg.sequence_number(),
                                self.timestamp(),
                                msg.timestamp(),
                            );
                            conn.write(&response).map_err(RastaError::from)?;
                        }
                    }
                    MessageType::Data => {
                        if self.connections.contains(&msg.sender()) {
                            println!("Received data from {}", msg.sender());
                            let seq_nr = msg.sequence_number();
                            let receiver = msg.sender();
                            let timestamp = msg.timestamp();
                            let response = if let Some(data) = (on_receive)(msg) {
                                Message::data_message(
                                    receiver,
                                    self.id,
                                    self.seq_nr.unwrap(),
                                    seq_nr,
                                    self.timestamp(),
                                    timestamp,
                                    data.as_ref(),
                                )
                            } else {
                                Message::heartbeat(
                                    receiver,
                                    self.id,
                                    self.seq_nr.unwrap(),
                                    seq_nr,
                                    self.timestamp(),
                                    timestamp,
                                )
                            };

                            conn.write(&response).map_err(RastaError::from)?;
                        }
                    }
                    MessageType::RetrData => unimplemented!("Handled by TCP"),
                }
            }
        }
        Ok(())
    }
}

/// This type roughly corresponds to [`std::net::TcpStream`].
/// Create it using [`RastaConnection::try_new`] and then handle
/// messages using [`RastaConnection::run`]. Alternatively, you
/// can manage the connection yourself. If you want to do this,
/// look at the implementation of [`RastaConnection::run`] for
/// inspiration.
pub struct RastaConnection {
    state: RastaConnectionState,
    id: RastaId,
    peer: RastaId,
    seq_nr: Option<u32>,
    confirmed_timestamp: u32,
    server: TcpStream,
}

impl RastaConnection {
    pub fn try_new<S: ToSocketAddrs>(server: S, id: RastaId) -> Result<Self, RastaError> {
        let connection = TcpStream::connect(server).map_err(RastaError::from)?;
        connection
            .set_read_timeout(Some(RASTA_TIMEOUT_DURATION))
            .map_err(RastaError::from)?;
        Ok(Self {
            state: RastaConnectionState::Down,
            id,
            peer: 0,
            seq_nr: None,
            confirmed_timestamp: 0,
            server: connection,
        })
    }

    fn next_seq_nr(&mut self) -> (u32, u32) {
        if let Some(seq_nr) = self.seq_nr {
            self.seq_nr.replace(seq_nr + 1);
            (seq_nr, seq_nr + 1)
        } else {
            self.seq_nr.replace(0);
            (0, 1)
        }
    }

    fn timestamp(&self) -> u32 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
    }

    pub fn open_connection(&mut self, receiver: u32) -> Result<(), RastaError> {
        println!("Sending connection request to {receiver}");
        let msg = Message::connection_request(receiver, self.id, self.timestamp(), N_SENDMAX);
        self.server.write(&msg).map_err(RastaError::from)?;
        let response = self.receive_message()?;
        let remote_version = &response.data()[0..4];
        if remote_version != &RASTA_VERSION {
            return Err(RastaError::VersionMismatch);
        }
        if response.message_type() == MessageType::ConnResp {
            self.state = RastaConnectionState::Up;
            self.seq_nr.replace(response.sequence_number());
            self.confirmed_timestamp = response.timestamp();
            self.peer = response.sender();
            println!(
                "Connected to {}",
                self.server.peer_addr().map_err(RastaError::from)?
            );
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
            self.server.write(&msg).map_err(RastaError::from)?;
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
        self.server.write(&msg).map_err(RastaError::from)?;
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
        self.server.write(&msg).map_err(RastaError::from)?;
        let response = self.receive_message()?;
        if response.message_type() == MessageType::HB {
            self.seq_nr.replace(response.sequence_number());
            self.confirmed_timestamp = response.timestamp();
        }
        Ok(())
    }

    pub fn connection_state_request(&self) -> RastaConnectionState {
        self.state
    }

    pub fn receive_message(&mut self) -> Result<Message, RastaError> {
        let mut buf = vec![0; 1024];
        let bytes_read = self.server.read(&mut buf).map_err(RastaError::from)?;
        Ok(Message::from(&buf[..bytes_read]))
    }

    pub fn run<F, D>(&mut self, peer: RastaId, mut message_fn: F) -> Result<(), RastaError>
    where
        F: FnMut(Option<Vec<u8>>) -> RastaCommand<D>,
        D: AsRef<[u8]>,
    {
        self.open_connection(peer)?;
        let mut previous_data = None;
        loop {
            match message_fn(previous_data.take()) {
                RastaCommand::Data(data) => {
                    self.send_data(data.as_ref())?;
                    let msg = self.receive_message()?;
                    if msg.message_type() == MessageType::Data {
                        previous_data.replace(Vec::from(msg.data()));
                    }
                }
                RastaCommand::Wait => {
                    self.send_heartbeat()?;
                    std::thread::sleep(RASTA_TIMEOUT_DURATION / 2);
                }
                RastaCommand::Disconnect => {
                    self.close_connection()?;
                    break;
                }
            }
        }
        Ok(())
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
