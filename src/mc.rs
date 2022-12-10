use crate::mc::packet_io::{PacketReadExt, PacketWriteExt, PartialVarInt, VarInt};
use crate::mc::pre_login::{
    Handshake, HandshakePacket, Listing, NextState, PingResponse, StatusPacket, StatusResponse,
};
use anyhow::{Context, Result};
use byteorder::{BigEndian, WriteBytesExt};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::{Read, Write};
use std::net::TcpStream;

pub mod packet_io;
pub mod pre_login;
pub mod text;

pub struct Connection {
    pub stream: TcpStream,
    packet: Option<PartialPacket>,
    definitely_modern: bool,

    pub state: ConnectionState,
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

    pub fn tick(
        &mut self,
        legacy_listing: impl FnOnce() -> Listing,
    ) -> Result<Vec<ReceivedPacket>> {
        let mut buf = [0; 1024];
        let bytes_read = self
            .stream
            .read(&mut buf)
            .context("failed to receive data from the client")?;

        let mut packets = vec![];
        let read = &buf[..bytes_read];
        for &byte in read {
            if !self.definitely_modern {
                if byte == 0xfe {
                    self.send_legacy_status_response(&read[1..], legacy_listing)
                        .context("failed to send a legacy status response")?;
                    // TODO: Close the connection
                    return Ok(vec![]);
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
                    let decoded = self
                        .state
                        .decode_packet(id, &mut slice)
                        .context("failed to decode the packet")?;
                    packets.push(decoded);
                }
                partial => self.packet = Some(partial),
            };
        }
        Ok(packets)
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

    pub fn send_legacy_status_response(
        &mut self,
        request: &[u8],
        listing: impl FnOnce() -> Listing,
    ) -> Result<()> {
        let listing = listing();
        let response = if request.is_empty() {
            // <1.4
            format!(
                "{}\u{00a7}{}\u{00a7}{}",
                listing.motd, listing.players.current, listing.players.max
            )
        } else {
            // 1.4-1.6
            format!(
                "\u{00a7}1\0{}\0{}\0{}\0{}\0{}",
                listing.version.value,
                listing.version.name,
                listing.motd,
                listing.players.current,
                listing.players.max
            )
        };

        let bytes = response
            .encode_utf16()
            .flat_map(u16::to_be_bytes)
            .collect::<Vec<u8>>();
        let len = bytes
            .len()
            .try_into()
            .context("the response length doesn't fit in a u16")?;

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

    pub fn handle_pre_play_packet(
        &mut self,
        packet: &ReceivedPacket,
        listing: impl FnOnce() -> Listing,
    ) -> Result<PacketHandleResult> {
        let result = match packet {
            ReceivedPacket::Handshake(packet) => {
                let HandshakePacket::Handshake(handshake) = packet;
                self.handle_handshake_packet(handshake)?;
                PacketHandleResult::Handled
            }
            ReceivedPacket::Status(packet) => {
                self.handle_status_packet(packet, listing)?;
                PacketHandleResult::Handled
            }
            _ => PacketHandleResult::Unhandled,
        };
        Ok(result)
    }

    fn handle_handshake_packet(&mut self, handshake: &Handshake) -> Result<()> {
        match handshake.next_state {
            NextState::Status => self.state = ConnectionState::Status,
            NextState::Login => self.state = ConnectionState::Login,
        }
        Ok(())
    }

    fn handle_status_packet(
        &mut self,
        packet: &StatusPacket,
        listing: impl FnOnce() -> Listing,
    ) -> Result<()> {
        match packet {
            StatusPacket::StatusRequest(_) => {
                let response = StatusResponse(listing());
                self.send_packet(response)
                    .context("failed to send a status response")?;
            }
            StatusPacket::PingRequest(request) => {
                let response = PingResponse(request.0);
                self.send_packet(response)
                    .context("failed to send a ping response")?;
                // TODO: Close the connection
            }
        }
        Ok(())
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
    pub fn decode_packet(&self, id: i32, buf: &mut impl Read) -> Result<ReceivedPacket> {
        let packet = match self {
            ConnectionState::Handshake => {
                ReceivedPacket::Handshake(HandshakePacket::decode(id, buf)?)
            }
            ConnectionState::Status => ReceivedPacket::Status(StatusPacket::decode(id, buf)?),
            ConnectionState::Login => todo!(),
            ConnectionState::Play => todo!(),
        };
        Ok(packet)
    }
}

pub enum ReceivedPacket {
    Handshake(HandshakePacket),
    Status(StatusPacket),
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

pub enum PacketHandleResult {
    Unhandled,
    Handled,
}

pub trait PacketFromServer {
    const ID: i32;

    fn write<W: Write>(&self, buf: &mut W) -> Result<()>;
}

pub trait PacketFromClient: Sized {
    const ID: i32;

    fn read<R: Read>(buf: &mut R) -> Result<Self>;
}

#[macro_export]
macro_rules! packets_from_client {
    ($enum_name:ident, $state_name:expr, [$($packet:ident),* $(,)?] $(,)?) => {
        // This will make IDEs treat $packet values primarily as types rather than enum variants
        // For auto-complete and more intuitive syntax highlighting
        #[allow(unused_parens)]
        const _: *const ($($packet),*) = ::std::ptr::null();

        pub enum $enum_name {
            $(
                $packet($packet),
            )*
        }

        impl $enum_name {
            #[allow(unreachable_code, unused_variables)]
            pub fn decode(id: i32, buf: &mut impl ::std::io::Read) -> ::anyhow::Result<Self> {
                let packet = match id {
                    $(
                        $packet::ID => Self::$packet($packet::read(buf)?),
                    )*
                    id => ::anyhow::bail!("invalid {} packet ID {id:#04x}", $state_name),
                };
                ::std::result::Result::Ok(packet)
            }
        }
    };
}
