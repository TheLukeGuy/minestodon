use crate::mc::packet_io::{PacketReadExt, PacketWriteExt};
use crate::mc::{Connection, ConnectionState, PacketFromClient, PacketFromServer};
use crate::{packets_from_client, ServerRef, ShouldClose};
use anyhow::{Context, Result};
use byteorder::{BigEndian, ReadBytesExt};
use log::info;
use std::io::{Read, Write};
use uuid::Uuid;

packets_from_client!(decode, "login", [LoginStart]);

pub struct LoginStart {
    pub name: String,
    pub signature: Option<Signature>,
    pub uuid: Option<Uuid>,
}

impl PacketFromClient for LoginStart {
    fn id() -> i32
    where
        Self: Sized,
    {
        0x00
    }

    fn read<R: Read>(buf: &mut R) -> Result<Self> {
        let name = buf.read_string().context("failed to read the username")?;

        let signature = buf
            .read_bool()
            .context("failed to read the boolean indicating the signature data")?;
        let signature = if signature {
            let signature = Signature::read(buf).context("failed to read the signature data")?;
            Some(signature)
        } else {
            None
        };

        let uuid = buf
            .read_bool()
            .context("failed to read the boolean indicating the UUID")?;
        let uuid = if uuid {
            let uuid = buf.read_uuid().context("failed to read the UUID")?;
            Some(uuid)
        } else {
            None
        };

        let packet = Self {
            name,
            signature,
            uuid,
        };
        Ok(packet)
    }

    fn handle(&self, connection: &mut Connection, _server: &ServerRef) -> Result<ShouldClose> {
        let uuid = Uuid::new_v4();
        info!("Assigning UUID {uuid} to player {}.", self.name);
        connection.uuid = Some(uuid);

        let compression = SetCompression(Connection::COMPRESSION_THRESHOLD);
        connection
            .send_packet(compression)
            .context("failed to send the desired compression threshold")?;
        connection.compressed = true;

        let success = LoginSuccess {
            uuid,
            name: self.name.clone(),
            properties: vec![],
        };
        connection
            .send_packet(success)
            .context("failed to send the login success packet")?;

        connection.set_state(ConnectionState::Play);
        Ok(ShouldClose::False)
    }
}

pub struct Signature {
    pub expiration_time: i64,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}

impl Signature {
    pub fn read<R: Read>(buf: &mut R) -> Result<Self> {
        let expiration_time = buf
            .read_i64::<BigEndian>()
            .context("failed to read the expiration time")?;

        let public_key_len = buf
            .read_var::<i32>()
            .context("failed to read the public key length")?
            .try_into()
            .context("the public key length doesn't fit in a usize")?;
        let mut public_key = vec![0; public_key_len];
        buf.read_exact(&mut public_key)
            .context("failed to read the public key")?;

        let signature_len = buf
            .read_var::<i32>()
            .context("failed to read the signature length")?
            .try_into()
            .context("the signature length doesn't fit in a usize")?;
        let mut signature = vec![0; signature_len];
        buf.read_exact(&mut signature)
            .context("failed to read the signature")?;

        let signature = Self {
            expiration_time,
            public_key,
            signature,
        };
        Ok(signature)
    }
}

pub struct SetCompression(i32);

impl PacketFromServer for SetCompression {
    fn id() -> i32 {
        0x03
    }

    fn write<W: Write>(&self, buf: &mut W) -> Result<()> {
        buf.write_var(self.0)
            .context("failed to write the compression threshold")
    }
}

pub struct LoginSuccess {
    pub uuid: Uuid,
    pub name: String,
    pub properties: Vec<LoginProperty>,
}

impl PacketFromServer for LoginSuccess {
    fn id() -> i32 {
        0x02
    }

    fn write<W: Write>(&self, buf: &mut W) -> Result<()> {
        buf.write_uuid(&self.uuid)
            .context("failed to write the UUID")?;
        buf.write_str(&self.name)
            .context("failed to write the username")?;

        let property_len = self
            .properties
            .len()
            .try_into()
            .context("the property length doesn't fit in an i32")?;
        buf.write_var::<i32>(property_len)
            .context("failed to write the property length")?;
        for property in &self.properties {
            property
                .write(buf)
                .context("failed to write the property")?;
        }

        Ok(())
    }
}

pub struct LoginProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

impl LoginProperty {
    pub fn write<W: Write>(&self, buf: &mut W) -> Result<()> {
        buf.write_str(&self.name)
            .context("failed to write the name")?;
        buf.write_str(&self.value)
            .context("failed to write the value")?;
        if let Some(signature) = &self.signature {
            buf.write_bool(true)
                .context("failed to write the boolean indicating the signature")?;
            buf.write_str(signature)
                .context("failed to write the signature")
        } else {
            buf.write_bool(false)
                .context("failed to write the boolean indicating the signature")
        }
    }
}