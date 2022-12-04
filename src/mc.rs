use crate::mc::packet_io::PacketWriteExt;
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

        let len = buf
            .len()
            .try_into()
            .context("the packet length doesn't fit in an i32")?;
        self.stream
            .write_var::<i32>(len)
            .context("failed to write the packet length")?;

        self.stream
            .write_all(&buf)
            .context("failed to write the packet body")
    }
}
