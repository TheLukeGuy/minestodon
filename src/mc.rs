use crate::mc::packet_io::{PacketReadExt, PacketWriteExt, PartialVarInt, VarInt};
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::{Read, Write};
use std::net::TcpStream;

pub mod packet_io;

pub struct Connection {
    pub stream: TcpStream,
    packet: Option<PartialPacket>,

    pub state: ConnectionState,
    pub compressed: bool,
}

impl Connection {
    pub const COMPRESSION_THRESHOLD: i32 = 256;

    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            packet: None,
            state: ConnectionState::Handshake,
            compressed: false,
        }
    }

    pub fn tick(&mut self) -> Result<Vec<ReceivedPacket>> {
        let mut buf = [0; 1024];
        let bytes_read = self
            .stream
            .read(&mut buf)
            .context("failed to receive data from the client")?;

        let mut packets = vec![];
        for &byte in &buf[..bytes_read] {
            let packet = self.packet.take().unwrap_or_else(PartialPacket::new);
            match packet.next(byte)? {
                PartialPacket::Full(body) => {
                    let body = if self.compressed {
                        let mut slice = &body[..];
                        let len = slice
                            .read_var::<i32>()
                            .context("failed to read the uncompressed packet length")?
                            .try_into()
                            .context("the uncompressed packet length doesn't fit in a usize")?;

                        if len != 0 {
                            let mut decoder = GzDecoder::new(slice);
                            let mut data = vec![0; len];
                            decoder
                                .read_exact(&mut data)
                                .context("failed to decode the packet data")?;
                            data
                        } else {
                            slice.to_vec()
                        }
                    } else {
                        body
                    };

                    let mut slice = &body[..];
                    let id = slice.read_var().context("failed to read the packet ID")?;
                    let decoded = self.state.decode_packet(id, &mut slice);
                    packets.push(decoded);
                }
                partial => self.packet = Some(partial),
            };
        }
        Ok(packets)
    }

    pub fn send_packet(&mut self, packet: impl PacketFromServer) -> Result<()> {
        let mut data_buf = Vec::with_capacity(1024);
        packet.write(&mut data_buf);

        let data_len = data_buf
            .len()
            .try_into()
            .context("the packet data length doesn't fit in an i32")?;

        let (len, buf) = if self.compressed {
            let mut buf = Vec::with_capacity(1024 + i32::MAX_VAR_LEN);
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

#[derive(Eq, PartialEq, Hash)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Play,
}

impl ConnectionState {
    pub fn decode_packet<R: Read>(&self, _id: i32, _buf: &mut R) -> ReceivedPacket {
        todo!("no packets have been implemented yet");
    }
}

pub enum ReceivedPacket {
    Handshake,
    DuringStatus,
    DuringLogin,
    DuringPlay,
}

enum PartialPacket {
    AwaitingLen(PartialVarInt<i32>),
    AwaitingBody { len: usize, body: Vec<u8> },
    Full(Vec<u8>),
}

impl PartialPacket {
    pub fn new() -> Self {
        Self::AwaitingLen(PartialVarInt::new())
    }

    pub fn next(self, byte: u8) -> Result<Self> {
        let next = match self {
            PartialPacket::AwaitingLen(len) => {
                let next = len
                    .next(byte)
                    .context("received an invalid byte while awaiting the packet length")?;
                match next {
                    PartialVarInt::Full(len) => {
                        let len = len
                            .try_into()
                            .context("the packet length doesn't fit in a usize")?;
                        Self::AwaitingBody { len, body: vec![] }
                    }
                    partial => Self::AwaitingLen(partial),
                }
            }
            PartialPacket::AwaitingBody { len, mut body } => {
                body.push(byte);
                if body.len() == len {
                    Self::Full(body)
                } else {
                    Self::AwaitingBody { len, body }
                }
            }
            full => full,
        };
        Ok(next)
    }
}

pub trait PacketFromServer {
    fn write<W: Write>(&self, buf: &mut W);
}

pub trait PacketFromClient {
    const STATE: ConnectionState;

    fn read<R: Read>(buf: &mut R) -> Self;
}
