use crate::mc::entity::GameMode;
use crate::mc::net::packet_io::PacketWriteExt;
use crate::mc::net::play::PluginMessageFromServer;
use crate::mc::net::{Connection, PacketFromServer};
use crate::mc::world::{
    Biome, BiomeEffects, BiomePrecipitation, BiomeWeather, BlockPos, DimensionEffects,
    DimensionType, InfiniteBurnTag, MonsterSettings,
};
use crate::mc::Identifier;
use crate::server::Server;
use anyhow::Context;
use anyhow::Result;
use byteorder::{BigEndian, WriteBytesExt};
use serde::Serialize;
use std::io::Write;

#[derive(Serialize)]
pub struct Registries {
    #[serde(rename = "minecraft:worldgen/biome")]
    pub biome: Registry,
    #[serde(rename = "minecraft:chat_type")]
    pub message_type: Registry,
    #[serde(rename = "minecraft:dimension_type")]
    pub dimension_type: Registry,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Registry {
    #[serde(rename = "minecraft:worldgen/biome")]
    Biome(Vec<RegistryEntry<Biome>>),
    #[serde(rename = "minecraft:chat_type")]
    MessageType(Vec<() /* TODO: Message types */>),
    #[serde(rename = "minecraft:dimension_type")]
    DimensionType(Vec<RegistryEntry<DimensionType>>),
}

impl Registry {}

#[derive(Serialize)]
pub struct RegistryEntry<T> {
    pub name: Identifier,
    pub id: i32,
    pub element: T,
}

pub struct PlayLogin {
    pub entity_id: i32,
    pub hardcore: bool,
    pub game_mode: GameMode,
    pub last_game_mode: Option<GameMode>,
    pub worlds: Vec<Identifier>,
    pub registries: Registries,
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

impl PacketFromServer for PlayLogin {
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
        worlds: vec![Identifier::minecraft("world")],
        registries: Registries {
            dimension_type: Registry::DimensionType(vec![RegistryEntry {
                name: Identifier::minecraft("overworld"),
                id: 0,
                element: DimensionType {
                    fixed_time: None,
                    sky_light: true,
                    ceiling: false,
                    ultra_warm: false,
                    natural: true,
                    coordinate_scale: 1.0,
                    bed_works: true,
                    respawn_anchor_works: false,
                    min_height: -64,
                    max_height: 384,
                    max_logical_height: 384,
                    infinite_burn_tag: InfiniteBurnTag::Overworld,
                    effects: DimensionEffects::Overworld,
                    ambient_light: 0.0,
                    monster_settings: MonsterSettings {
                        piglin_safe: false,
                        raids: true,
                        monster_spawn_light_level: 0,
                        monster_spawn_block_light_limit: 0,
                    },
                },
            }]),
            biome: Registry::Biome(vec![RegistryEntry {
                name: Identifier::minecraft("plains"),
                id: 0,
                element: Biome {
                    weather: BiomeWeather {
                        precipitation: BiomePrecipitation::Rain,
                        temperature: 0.8,
                        temperature_modifier: None,
                        downfall: 0.4,
                    },
                    effects: BiomeEffects {
                        fog_color: 0xc0d8ff,
                        water_color: 0x3f76e4,
                        water_fog_color: 0x050533,
                        sky_color: 0x78a7ff,
                        foliage_color: None,
                        grass_color: None,
                        grass_color_modifier: None,
                    },
                },
            }]),
            message_type: Registry::MessageType(vec![]),
        },
        dimension_type: Identifier::minecraft("overworld"),
        world: Identifier::minecraft("world"),
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

    let brand = PluginMessageFromServer::brand("Minestodon");
    connection
        .send_packet(brand)
        .context("failed to send the server brand")
}
