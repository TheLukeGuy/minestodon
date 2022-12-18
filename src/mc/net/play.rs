use crate::mc::entity::GameMode;
use crate::mc::net::packet_io::PacketWriteExt;
use crate::mc::net::{Connection, PacketFromServer};
use crate::mc::world::BlockPos;
use crate::mc::Identifier;
use crate::server::Server;
use anyhow::{Context, Result};
use byteorder::{BigEndian, WriteBytesExt};
use fastnbt::nbt;
use std::borrow::Cow;
use std::io::Write;

pub fn init_play(connection: &mut Connection, server: &Server) -> Result<()> {
    let join_game = JoinGame {
        entity_id: server.next_entity_id(),
        hardcore: false,
        game_mode: GameMode::Adventure,
        last_game_mode: None,
        dimensions: vec![Identifier::minecraft("world")],
        registries: nbt!({
            "minecraft:dimension_type": {
                "type": "minecraft:dimension_type",
                "value": [
                    {
                        "name": "minecraft:overworld",
                        "id": 0,
                        "element": {
                            "piglin_safe": 0u8,
                            "has_raids": 1u8,
                            "monster_spawn_light_level": 15,
                            "monster_spawn_block_light_limit": {
                                "type": "minecraft:uniform",
                                "value": {
                                    "min_inclusive": 0,
                                    "max_inclusive": 7,
                                },
                            },
                            "natural": 1u8,
                            "ambient_light": 0.0,
                            "infiniburn": "#minecraft:infiniburn_overworld",
                            "respawn_anchor_works": 0u8,
                            "has_skylight": 1u8,
                            "bed_works": 1u8,
                            "effects": "minecraft:overworld",
                            "min_y": -64,
                            "height": 384,
                            "logical_height": 384,
                            "coordinate_scale": 1.0f64,
                            "ultrawarm": 0u8,
                            "has_ceiling": 0u8,
                        },
                    },
                ],
            },
            "minecraft:worldgen/biome": {
                "type": "minecraft:worldgen/biome",
                "value": [],
            },
            "minecraft:chat_type": {
                "type": "minecraft:chat_type",
                "value": [],
            },
        }),
        dimension_type: Identifier::minecraft("overworld"),
        dimension: Identifier::minecraft("world"),
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
        .send_packet(join_game)
        .context("failed to send the join game packet")?;

    let brand = PluginMessageFromServer::brand("Minestodon");
    connection
        .send_packet(brand)
        .context("failed to send the server brand")
}

pub struct JoinGame {
    pub entity_id: i32,
    pub hardcore: bool,
    pub game_mode: GameMode,
    pub last_game_mode: Option<GameMode>,
    pub dimensions: Vec<Identifier>,
    pub registries: fastnbt::Value,
    pub dimension_type: Identifier,
    pub dimension: Identifier,
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

impl PacketFromServer for JoinGame {
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

        let dimension_len = self
            .dimensions
            .len()
            .try_into()
            .context("the dimension length doesn't fit in an i32")?;
        buf.write_var::<i32>(dimension_len)
            .context("failed to write the dimension length")?;
        for dimension in &self.dimensions {
            buf.write_identifier(dimension)
                .context("failed to write the dimension")?;
        }

        buf.write_nbt(&self.registries)
            .context("failed to write the registries")?;
        buf.write_identifier(&self.dimension_type)
            .context("failed to write the dimension type")?;
        buf.write_identifier(&self.dimension)
            .context("failed to write the current dimension")?;
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
