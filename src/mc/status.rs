use crate::mc::packet_io::PacketWriteExt;
use crate::mc::{PacketFromClient, PacketFromServer};
use crate::packets_from_client;
use anyhow::{Context, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use serde_json::Value;
use std::io::{Read, Write};
use uuid::Uuid;

#[derive(Serialize)]
pub struct Listing {
    pub version: ListingVersion,
    pub players: ListingPlayers,
    #[serde(rename = "description")]
    pub motd: Value,
    #[serde(rename = "favicon")]
    pub icon: String,
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
    pub sample: Vec<ListingPlayer>,
}

#[derive(Serialize)]
pub struct ListingPlayer {
    pub name: String,
    pub id: Uuid,
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
