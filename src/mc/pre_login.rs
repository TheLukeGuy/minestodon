use crate::mc::packet_io::{PacketReadExt, PacketWriteExt};
use crate::mc::text::Text;
use crate::mc::{Connection, ConnectionState, PacketFromClient, PacketFromServer};
use crate::packets_from_client;
use crate::server::{ServerRef, ShouldClose};
use anyhow::{Context, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num_enum::TryFromPrimitive;
use serde::Serialize;
use std::io::{Read, Write};
use uuid::Uuid;

#[derive(Serialize)]
pub struct Listing {
    pub version: ListingVersion,
    pub players: ListingPlayers,
    #[serde(rename = "description")]
    pub motd: Text,
    #[serde(rename = "favicon", skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Serialize)]
pub struct ListingVersion {
    #[serde(rename = "protocol")]
    pub value: i32,
    #[serde(rename = "name")]
    pub name: String,
}

#[derive(Serialize)]
pub struct ListingPlayers {
    #[serde(rename = "online")]
    pub current: i32,
    pub max: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample: Option<Vec<ListingPlayer>>,
}

#[derive(Serialize)]
pub struct ListingPlayer {
    pub name: String,
    pub id: Uuid,
}

packets_from_client!(decode_handshake, "handshake", [Handshake]);

pub struct Handshake {
    pub version: i32,
    pub server_addr: String,
    pub server_port: u16,
    pub next_state: NextState,
}

impl PacketFromClient for Handshake {
    fn id() -> i32 {
        0x00
    }

    fn read<R: Read>(buf: &mut R) -> Result<Self> {
        let version = buf.read_var().context("failed to read the version")?;
        let server_addr = buf
            .read_string()
            .context("failed to read the server address")?;
        let server_port = buf
            .read_u16::<BigEndian>()
            .context("failed to read the server port")?;
        let next_state = buf
            .read_var::<i32>()
            .context("failed to read the next state")?
            .try_into()
            .context("the next state is invalid, it must be 1 or 2")?;

        let packet = Self {
            version,
            server_addr,
            server_port,
            next_state,
        };
        Ok(packet)
    }

    fn handle(&self, connection: &mut Connection, _server: &ServerRef) -> Result<ShouldClose> {
        match self.next_state {
            NextState::Status => connection.set_state(ConnectionState::Status),
            NextState::Login => connection.set_state(ConnectionState::Login),
        }
        Ok(ShouldClose::False)
    }
}

#[derive(TryFromPrimitive)]
#[repr(i32)]
pub enum NextState {
    Status = 1,
    Login = 2,
}

packets_from_client!(decode_status, "status", [StatusRequest, PingRequest]);

pub struct StatusRequest;

impl PacketFromClient for StatusRequest {
    fn id() -> i32 {
        0x00
    }

    fn read<R: Read>(_buf: &mut R) -> Result<Self> {
        Ok(Self)
    }

    fn handle(&self, connection: &mut Connection, server: &ServerRef) -> Result<ShouldClose> {
        let response = StatusResponse(server.listing());
        connection
            .send_packet(response)
            .context("failed to send a status response packet")?;
        Ok(ShouldClose::False)
    }
}

pub struct StatusResponse(pub Listing);

impl PacketFromServer for StatusResponse {
    fn id() -> i32 {
        0x00
    }

    fn write<W: Write>(&self, buf: &mut W) -> Result<()> {
        let serialized =
            serde_json::to_string(&self.0).context("failed to serialize the server listing")?;
        buf.write_str(&serialized)
            .context("failed to write the response")
    }
}

pub struct PingRequest(pub i64);

impl PacketFromClient for PingRequest {
    fn id() -> i32 {
        0x01
    }

    fn read<R: Read>(buf: &mut R) -> Result<Self> {
        let payload = buf
            .read_i64::<BigEndian>()
            .context("failed to read the payload")?;
        Ok(Self(payload))
    }

    fn handle(&self, connection: &mut Connection, _server: &ServerRef) -> Result<ShouldClose> {
        let response = PingResponse(self.0);
        connection
            .send_packet(response)
            .context("failed to send a ping response packet")?;
        Ok(ShouldClose::True)
    }
}

pub struct PingResponse(pub i64);

impl PacketFromServer for PingResponse {
    fn id() -> i32 {
        0x01
    }

    fn write<W: Write>(&self, buf: &mut W) -> Result<()> {
        buf.write_i64::<BigEndian>(self.0)
            .context("failed to write the payload")
    }
}
