use anyhow::{Context, Result};
use std::io::Write;
use std::net::TcpStream;

pub mod packet_io;

pub struct Connection {
    pub stream: TcpStream,
}

impl Connection {
    pub fn send_packet(&mut self, write: impl FnOnce(&mut Vec<u8>)) -> Result<()> {
        let mut buf = Vec::with_capacity(1024);
        write(&mut buf);
        self.stream
            .write_all(&buf)
            .context("failed to send a packet to the client")
    }
}
