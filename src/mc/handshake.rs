use crate::mc::packet_io::PacketReadExt;
use crate::mc::status::Listing;
use crate::mc::{PacketFromClient, PacketFromServer};
use crate::packets_from_client;
use anyhow::{bail, Context, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num_enum::TryFromPrimitive;
use std::io::{Read, Write};

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

pub struct LegacyPingResponse(Listing);

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
            .context("the response length doesn't fit in an i32")?;
        buf.write_u16::<BigEndian>(len)
            .context("failed to write the response length")?;
        buf.write_all(&bytes)
            .context("failed to write the response")?;

        Ok(())
    }
}
