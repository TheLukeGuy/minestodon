use crate::mc::packet_io::{PacketReadExt, PacketWriteExt};
use crate::mc::text::Text;
use crate::mc::{PacketFromClient, PacketFromServer};
use crate::packets_from_client;
use anyhow::{bail, Context, Result};
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

packets_from_client!(HandshakePacket, "handshake", [Handshake, LegacyPingRequest]);

pub struct Handshake {
    pub version: i32,
    pub server_addr: String,
    pub server_port: u16,
    pub next_state: NextState,
}

impl PacketFromClient for Handshake {
    const ID: i32 = 0x00;

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
}

#[derive(TryFromPrimitive)]
#[repr(i32)]
pub enum NextState {
    Status = 1,
    Login = 2,
}

pub struct LegacyPingRequest;

impl PacketFromClient for LegacyPingRequest {
    const ID: i32 = 0xfe;

    fn read<R: Read>(buf: &mut R) -> Result<Self> {
        let next_two_bytes = buf
            .read_u16::<BigEndian>()
            .context("failed to read the next two bytes")?;
        if next_two_bytes != 0x01fa {
            bail!("invalid legacy ping request");
        }
        Ok(Self)
    }
}

pub struct LegacyPingResponse(pub Listing);

impl LegacyPingResponse {
    pub fn response_string(&self) -> String {
        format!(
            "\u{00a7}1\0{}\0{}\0{}\0{}\0{}",
            self.0.version.value,
            self.0.version.name,
            self.0.motd,
            self.0.players.current,
            self.0.players.max
        )
    }
}

impl PacketFromServer for LegacyPingResponse {
    const ID: i32 = 0xff;

    fn write<W: Write>(&self, buf: &mut W) -> Result<()> {
        let response = self.response_string();
        let bytes = response
            .encode_utf16()
            .flat_map(u16::to_be_bytes)
            .collect::<Vec<u8>>();

        let len = bytes
            .len()
            .try_into()
            .context("the response length doesn't fit in a u16")?;
        buf.write_u16::<BigEndian>(len)
            .context("failed to write the response length")?;
        buf.write_all(&bytes)
            .context("failed to write the response")?;

        Ok(())
    }
}

packets_from_client!(StatusPacket, "status", [StatusRequest, PingRequest]);

pub struct StatusRequest;

impl PacketFromClient for StatusRequest {
    const ID: i32 = 0x00;

    fn read<R: Read>(_buf: &mut R) -> Result<Self> {
        Ok(Self)
    }
}

pub struct StatusResponse(pub Listing);

impl PacketFromServer for StatusResponse {
    const ID: i32 = 0x00;

    fn write<W: Write>(&self, buf: &mut W) -> Result<()> {
        let serialized =
            serde_json::to_string(&self.0).context("failed to serialize the server listing")?;
        buf.write_str(&serialized)
            .context("failed to write the response")
    }
}

pub struct PingRequest(pub i64);

impl PacketFromClient for PingRequest {
    const ID: i32 = 0x01;

    fn read<R: Read>(buf: &mut R) -> Result<Self> {
        let payload = buf
            .read_i64::<BigEndian>()
            .context("failed to read the payload")?;
        Ok(Self(payload))
    }
}

pub struct PingResponse(pub i64);

impl PacketFromServer for PingResponse {
    const ID: i32 = 0x01;

    fn write<W: Write>(&self, buf: &mut W) -> Result<()> {
        buf.write_i64::<BigEndian>(self.0)
            .context("failed to write the payload")
    }
}
