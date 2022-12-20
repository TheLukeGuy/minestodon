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
    pub fn brand<'a, I>(name: I) -> Self
    where
        I: Into<Cow<'a, str>>,
    {
        let channel = Identifier::minecraft("brand");
        let data = name.into().into_owned().into_bytes();
        Self { channel, data }
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
