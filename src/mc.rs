use crate::mc::packet_io::{PacketReadExt, PacketWriteExt, PartialVarInt, VarInt};
use crate::mc::pre_login::Listing;
use crate::{ServerRef, ShouldClose};
use anyhow::{Context, Result};
use byteorder::{BigEndian, WriteBytesExt};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::debug;
use std::io::{Read, Write};
use std::net::TcpStream;

pub mod packet_io;
pub mod pre_login;
pub mod text;

pub struct Connection {
    pub stream: TcpStream,
    packet: Option<PartialPacket>,
    definitely_modern: bool,

    state: ConnectionState,
    pub compressed: bool,
}

impl Connection {
    pub const COMPRESSION_THRESHOLD: i32 = 256;

    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            packet: None,
            definitely_modern: false,
            state: ConnectionState::Handshake,
            compressed: false,
        }
    }

    pub fn tick(&mut self, server: &ServerRef) -> Result<ShouldClose> {
        let mut buf = [0; 1024];
        let bytes_read = self
            .stream
            .read(&mut buf)
            .context("failed to receive data from the client")?;

        let read = &buf[..bytes_read];
        for &byte in read {
            if !self.definitely_modern {
                if byte == 0xfe {
                    self.send_legacy_status_response(&read[1..], server.legacy_listing())
                        .context("failed to send a legacy status response")?;
                    return Ok(ShouldClose::True);
                } else {
                    self.definitely_modern = true;
                }
            }

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
                    let close = self
                        .decode_and_handle_packet(id, &mut slice, server)
                        .context("failed to decode and handle the packet")?;
                    if close.is_true() {
                        return Ok(ShouldClose::True);
                    }
                }
                partial => self.packet = Some(partial),
            };
        }
        Ok(ShouldClose::False)
    }

    pub fn decode_and_handle_packet(
        &mut self,
        id: i32,
        buf: &mut impl Read,
        server: &ServerRef,
    ) -> Result<ShouldClose> {
        let decoded = match self.state {
            ConnectionState::Handshake => pre_login::decode_handshake(id, buf),
            ConnectionState::Status => pre_login::decode_status(id, buf),
            ConnectionState::Login => todo!(),
            ConnectionState::Play => todo!(),
        };
        let decoded = decoded.context("failed to decode the packet")?;
        decoded
            .handle(self, server)
            .context("failed to handle the packet")
    }

    pub fn send_packet<P: PacketFromServer>(&mut self, packet: P) -> Result<()> {
        let mut data_buf = Vec::with_capacity(1024);
        data_buf
            .write_var(P::ID)
            .context("failed to write the packet ID")?;
        packet
            .write(&mut data_buf)
            .context("failed to write the packet data")?;

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

    pub fn send_legacy_status_response(&mut self, request: &[u8], listing: Listing) -> Result<()> {
        let response = if request.is_empty() {
            // <1.4
            debug!("Sending a legacy (<1.4) status response.");
            format!(
                "{}\u{00a7}{}\u{00a7}{}",
                listing.motd.to_plain_text(),
                listing.players.current,
                listing.players.max
            )
        } else {
            // 1.4-1.6
            debug!("Sending a legacy (1.4-1.6) status response.");
            format!(
                "\u{00a7}1\0{}\0{}\0{}\0{}\0{}",
                listing.version.value,
                listing.version.name,
                listing.motd.to_legacy_text(),
                listing.players.current,
                listing.players.max
            )
        };

        let len = response
            .chars()
            .count()
            .try_into()
            .context("the response length doesn't fit in a u16")?;
        let bytes = response
            .encode_utf16()
            .flat_map(u16::to_be_bytes)
            .collect::<Vec<u8>>();

        self.stream
            .write_u8(0xff)
            .context("failed to send the packet ID")?;
        self.stream
            .write_u16::<BigEndian>(len)
            .context("failed to send the response length")?;
        self.stream
            .write_all(&bytes)
            .context("failed to send the response")?;

        Ok(())
    }

    pub fn set_state(&mut self, state: ConnectionState) {
        debug!("State change: {:?} -> {state:?}", self.state);
        self.state = state;
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Play,
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
            Self::AwaitingLen(len) => {
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
            Self::AwaitingBody { len, mut body } => {
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
    const ID: i32;

    fn write<W: Write>(&self, buf: &mut W) -> Result<()>;
}

pub trait PacketFromClient {
    fn id() -> i32
    where
        Self: Sized;

    fn read<R: Read>(buf: &mut R) -> Result<Self>
    where
        Self: Sized;

    fn handle(&self, connection: &mut Connection, server: &ServerRef) -> Result<ShouldClose>;
}

#[macro_export]
macro_rules! packets_from_client {
    ($fn_name:ident, $state:expr, [$($packet:ident),* $(,)?] $(,)?) => {
        #[allow(unreachable_code, unused_variables)]
        pub fn $fn_name(
            id: i32,
            buf: &mut impl ::std::io::Read,
        ) -> ::anyhow::Result<::std::boxed::Box<dyn $crate::mc::PacketFromClient>> {
            let packet: ::std::boxed::Box<dyn $crate::mc::PacketFromClient> = match id {
                $(
                    id if id == $packet::id() => ::std::boxed::Box::new($packet::read(buf)?),
                )*
                id => ::anyhow::bail!(::std::concat!("invalid ", $state, " packet ID {:#04x}"), id),
            };
            ::std::result::Result::Ok(packet)
        }
    };
}
