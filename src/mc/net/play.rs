use crate::mc::net::packet_io::PacketWriteExt;
use crate::mc::net::PacketFromServer;
use crate::mc::Identifier;
use anyhow::{Context, Result};
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
            channel: Identifier::minecraft("brand"),
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
