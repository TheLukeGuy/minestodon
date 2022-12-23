use crate::mc::net::packet_io::PacketWriteExt;
use crate::mc::net::PacketFromServer;
use crate::mc::world::BlockPos;
use crate::mc::Identifier;
use anyhow::{Context, Result};
use byteorder::{BigEndian, WriteBytesExt};
use minestodon_macros::minecraft;
use std::borrow::Cow;
use std::io::Write;

pub mod setup;

pub struct PluginMessageFromServer {
    pub channel: Identifier,
    pub data: Vec<u8>,
}

impl PluginMessageFromServer {
    pub fn brand<'a, I>(name: I) -> Result<Self>
    where
        I: Into<Cow<'a, str>>,
    {
        let name = name.into();
        let mut data = Vec::with_capacity(name.len());
        data.write_str(&name)
            .context("failed to write the brand name")?;

        let packet = Self {
            channel: minecraft!("brand"),
            data,
        };
        Ok(packet)
    }
}

impl PacketFromServer for PluginMessageFromServer {
    fn id() -> i32 {
        0x15
    }

    fn write<W: Write>(&self, buf: &mut W) -> Result<()> {
        buf.write_identifier(&self.channel)
            .context("failed to write the channel")?;
        buf.write_all(&self.data)
            .context("failed to write the data")
    }
}

pub struct SyncPlayerPos {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: u8,
    pub teleport_id: i32,
    pub dismount_vehicle: bool,
}

impl PacketFromServer for SyncPlayerPos {
    fn id() -> i32 {
        0x38
    }

    fn write<W: Write>(&self, buf: &mut W) -> Result<()> {
        buf.write_f64::<BigEndian>(self.x)
            .context("failed to write the X position")?;
        buf.write_f64::<BigEndian>(self.y)
            .context("failed to write the Y position")?;
        buf.write_f64::<BigEndian>(self.z)
            .context("failed to write the Z position")?;
        buf.write_f32::<BigEndian>(self.yaw)
            .context("failed to write the yaw")?;
        buf.write_f32::<BigEndian>(self.pitch)
            .context("failed to write the pitch")?;
        buf.write_u8(self.flags)
            .context("failed to write the flags")?;
        buf.write_var(self.teleport_id)
            .context("failed to write the teleport ID")?;
        buf.write_bool(self.dismount_vehicle)
            .context("failed to write the vehicle dismount indicator")
    }
}

pub struct SetSpawnPos {
    pub pos: BlockPos,
    pub angle: f32,
}

impl PacketFromServer for SetSpawnPos {
    fn id() -> i32 {
        0x4c
    }

    fn write<W: Write>(&self, buf: &mut W) -> Result<()> {
        buf.write_block_pos(&self.pos)
            .context("failed to write the position")?;
        buf.write_f32::<BigEndian>(self.angle)
            .context("failed to write the angle")
    }
}
