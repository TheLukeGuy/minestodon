use crate::mc::packet_io::PacketWriteExt;
use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;
use std::net::TcpStream;

pub mod packet_io;

pub struct Connection {
    pub stream: TcpStream,
    pub compressed: bool,
}

impl Connection {
    pub const COMPRESSION_THRESHOLD: i32 = 256;

    pub fn send_packet(&mut self, write: impl FnOnce(&mut Vec<u8>)) -> Result<()> {
        let mut data_buf = Vec::with_capacity(1024);
        write(&mut data_buf);

        let data_len = data_buf
            .len()
            .try_into()
            .context("the packet data length doesn't fit in an i32")?;

        let (len, buf) = if self.compressed {
            let mut buf = Vec::with_capacity(1024 + 10);
            if data_len >= Self::COMPRESSION_THRESHOLD {
                buf.write_var(data_len)
                    .context("failed to write the uncompressed packet length")?;

                let mut encoder = GzEncoder::new(buf, Compression::default());
                encoder
                    .write_all(&data_buf)
                    .context("failed to encode and write the packet data")?;
                buf = encoder
                    .finish()
                    .context("failed to finish encoding the packet data")?;
            } else {
                buf.write_var(0)
                    .context("failed to write a zero to indicate uncompressed packet data")?;
                buf.write_all(&data_buf)
                    .context("failed to write the packet data")?;
            }

            let len = buf
                .len()
                .try_into()
                .context("the packet length doesn't fit in an i32")?;
            (len, buf)
        } else {
            (data_len, data_buf)
        };

        self.stream
            .write_var::<i32>(len)
            .context("failed to send the packet length")?;
        self.stream
            .write_all(&buf)
            .context("failed to send the packet body")
    }
}
