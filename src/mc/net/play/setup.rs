use crate::mc::entity::GameMode;
use crate::mc::net::packet_io::PacketWriteExt;
use crate::mc::net::play::PluginMessageFromServer;
use crate::mc::net::{Connection, PacketFromServer};
use crate::mc::registry::Registry;
use crate::mc::world::{Biome, BlockPos, DimensionType};
use crate::mc::{registry, world, Identifier};
use crate::server::Server;
use anyhow::Context;
use anyhow::Result;
use byteorder::{BigEndian, WriteBytesExt};
use minestodon_macros::minestodon;
use serde::Serialize;
use std::io::Write;

#[derive(Serialize)]
pub struct Registries<'a> {
    #[serde(rename = "minecraft:worldgen/biome")]
    pub biome: &'a Registry<Biome>,
    #[serde(rename = "minecraft:chat_type")]
    pub message_type: &'a Registry<()>,
    #[serde(rename = "minecraft:dimension_type")]
    pub dimension_type: &'a Registry<DimensionType>,
}

pub struct PlayLogin<'a> {
    pub entity_id: i32,
    pub hardcore: bool,
    pub game_mode: GameMode,
    pub last_game_mode: Option<GameMode>,
    pub worlds: Vec<Identifier>,
    pub registries: Registries<'a>,
    pub dimension_type: Identifier,
    pub world: Identifier,
    pub hashed_seed: i64,
    pub max_players: i32,
    pub view_distance: i32,
    pub simulation_distance: i32,
    pub reduced_debug_info: bool,
    pub respawn_screen: bool,
    pub debug_mode: bool,
    pub flat_world: bool,
    pub death_pos: Option<(Identifier, BlockPos)>,
}

impl PacketFromServer for PlayLogin<'_> {
    fn id() -> i32 {
        0x24
    }

    fn write<W: Write>(&self, buf: &mut W) -> Result<()> {
        buf.write_i32::<BigEndian>(self.entity_id)
            .context("failed to write the entity ID")?;
        buf.write_bool(self.hardcore)
            .context("failed to write the hardcore indicator")?;
        buf.write_i8(self.game_mode.into())
            .context("failed to write the game mode")?;
        buf.write_i8(self.last_game_mode.map(GameMode::into).unwrap_or(-1))
            .context("failed to write the last game mode")?;

        let world_len = self
            .worlds
            .len()
            .try_into()
            .context("the world count doesn't fit in an i32")?;
        buf.write_var::<i32>(world_len)
            .context("failed to write the world count")?;
        for dimension in &self.worlds {
            buf.write_identifier(dimension)
                .context("failed to write the world")?;
        }

        buf.write_nbt(&self.registries)
            .context("failed to write the registries")?;
        buf.write_identifier(&self.dimension_type)
            .context("failed to write the dimension type")?;
        buf.write_identifier(&self.world)
            .context("failed to write the current world")?;
        buf.write_i64::<BigEndian>(self.hashed_seed)
            .context("failed to write the hashed seed")?;
        buf.write_var(self.max_players)
            .context("failed to write the maximum player count")?;
        buf.write_var(self.view_distance)
            .context("failed to write the view distance")?;
        buf.write_var(self.simulation_distance)
            .context("failed to write the simulation distance")?;
        buf.write_bool(self.reduced_debug_info)
            .context("failed to write the reduced debug info indicator")?;
        buf.write_bool(self.respawn_screen)
            .context("failed to write the respawn screen indicator")?;
        buf.write_bool(self.debug_mode)
            .context("failed to write the debug mode indicator")?;
        buf.write_bool(self.flat_world)
            .context("failed to write the flat world indicator")?;

        buf.write_bool(self.death_pos.is_some())
            .context("failed to write the death position indicator")?;
        if let Some((dimension, pos)) = &self.death_pos {
            buf.write_identifier(dimension)
                .context("failed to write the death dimension")?;
            buf.write_block_pos(pos)
                .context("failed to write the death position")?;
        }

        Ok(())
    }
}

pub fn set_up(connection: &mut Connection, server: &Server) -> Result<()> {
    let login = PlayLogin {
        entity_id: server.next_entity_id(),
        hardcore: false,
        game_mode: GameMode::Adventure,
        last_game_mode: None,
        worlds: vec![minestodon!("world")],
        registries: Registries {
            biome: &registry::BIOMES,
            message_type: &registry::MESSAGE_TYPES,
            dimension_type: &registry::DIMENSION_TYPES,
        },
        dimension_type: world::DIMENSION_TYPE,
        world: minestodon!("world"),
        hashed_seed: 0,
        max_players: 0,
        view_distance: 32,
        simulation_distance: 32,
        reduced_debug_info: false,
        respawn_screen: true,
        debug_mode: false,
        flat_world: true,
        death_pos: None,
    };
    connection
        .send_packet(login)
        .context("failed to send the login packet")?;

    let brand = PluginMessageFromServer::brand("Minestodon")
        .context("failed to create the server brand plugin message")?;
    connection
        .send_packet(brand)
        .context("failed to send the server brand")
}
