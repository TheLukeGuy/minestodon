use crate::mc::net::login::{LoginSuccess, SetCompression};
use crate::mc::net::play::setup;
use crate::mc::net::{Connection, ConnectionState};
use crate::server::Server;
use anyhow::{Context, Result};
use log::info;
use num_enum::IntoPrimitive;
use uuid::Uuid;

pub struct Player {
    pub connection: Connection,
    pub server: Server,
    uuid: Uuid,

    pub username: String,
}

impl Player {
    pub fn new(connection: Connection, username: String, server: Server) -> Self {
        let uuid = Uuid::new_v4();
        info!("Assigning UUID {uuid} to player {}.", username);

        Self {
            connection,
            server,
            uuid,
            username,
        }
    }

    pub fn finish_joining(&mut self) -> Result<()> {
        let compression = SetCompression(Connection::COMPRESSION_THRESHOLD);
        self.connection
            .send_packet(compression)
            .context("failed to send the desired compression threshold")?;
        self.connection.compressed = true;

        let success = LoginSuccess {
            uuid: self.uuid,
            name: self.username.clone(),
            properties: vec![],
        };
        self.connection
            .send_packet(success)
            .context("failed to send the login success packet")?;

        self.connection.set_state(ConnectionState::Play);
        setup::set_up(&mut self.connection, &self.server)
            .context("failed to set up after login")?;
        Ok(())
    }

    pub fn tick(&mut self, _server: &Server) -> Result<()> {
        Ok(())
    }
}

#[derive(Copy, Clone, IntoPrimitive)]
#[repr(i8)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}
