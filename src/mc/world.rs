use crate::mc::registry::Registry;
use crate::mc::Identifier;
use minestodon_macros::{minecraft, minestodon};
use serde::Serialize;

pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Serialize)]
pub struct Biome {
    #[serde(flatten)]
    pub weather: BiomeWeather,
    pub effects: BiomeEffects,
}

#[derive(Serialize)]
pub struct BiomeWeather {
    pub precipitation: BiomePrecipitation,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature_modifier: Option<BiomeTemperatureModifier>,
    pub downfall: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BiomePrecipitation {
    None,
    Rain,
    Snow,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BiomeTemperatureModifier {
    None,
    Frozen,
}

#[derive(Serialize)]
pub struct BiomeEffects {
    pub fog_color: i32,
    pub water_color: i32,
    pub water_fog_color: i32,
    pub sky_color: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foliage_color: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grass_color: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grass_color_modifier: Option<BiomeGrassColorModifier>,
    // TODO: Particles and sounds
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BiomeGrassColorModifier {
    None,
    DarkForest,
    Swamp,
}

#[derive(Serialize)]
pub struct DimensionType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_time: Option<i64>,
    #[serde(rename = "has_skylight")]
    pub sky_light: bool,
    #[serde(rename = "has_ceiling")]
    pub ceiling: bool,
    #[serde(rename = "ultrawarm")]
    pub ultra_warm: bool,
    pub natural: bool,
    pub coordinate_scale: f64,
    pub bed_works: bool,
    pub respawn_anchor_works: bool,
    #[serde(rename = "min_y")]
    pub min_height: i32,
    #[serde(rename = "height")]
    pub max_height: i32,
    #[serde(rename = "logical_height")]
    pub max_logical_height: i32,
    #[serde(rename = "infiniburn")]
    pub infinite_burn_tag: InfiniteBurnTag,
    pub effects: DimensionEffects,
    pub ambient_light: f32,
    #[serde(flatten)]
    pub monster_settings: MonsterSettings,
}

#[derive(Serialize)]
pub enum DimensionEffects {
    #[serde(rename = "minecraft:overworld")]
    Overworld,
    #[serde(rename = "minecraft:the_nether")]
    Nether,
    #[serde(rename = "minecraft:the_end")]
    End,
}

#[derive(Serialize)]
pub enum InfiniteBurnTag {
    #[serde(rename = "#minecraft:infiniburn_overworld")]
    Overworld,
    #[serde(rename = "#minecraft:infiniburn_nether")]
    Nether,
    #[serde(rename = "#minecraft:infiniburn_end")]
    End,
}

#[derive(Serialize)]
pub struct MonsterSettings {
    pub piglin_safe: bool,
    #[serde(rename = "has_raids")]
    pub raids: bool,
    pub monster_spawn_light_level: i32,
    pub monster_spawn_block_light_limit: i32,
}

pub const BIOME: Identifier = minestodon!("tootlands");

pub fn register_biomes(registry: &Registry<Biome>) {
    registry.register(
        BIOME,
        Biome {
            weather: BiomeWeather {
                precipitation: BiomePrecipitation::Snow,
                temperature: 0.0,
                temperature_modifier: None,
                downfall: 0.5,
            },
            effects: BiomeEffects {
                fog_color: 0xc0d8ff,
                water_color: 0x3f76e4,
                water_fog_color: 0x050533,
                sky_color: 0x050533,
                foliage_color: None,
                grass_color: None,
                grass_color_modifier: None,
            },
        },
    );

    // Clients will disconnect with an error if we don't send the plains biome
    registry.register(
        minecraft!("plains"),
        Biome {
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
    );
}

pub const DIMENSION_TYPE: Identifier = minestodon!("fediverse");

pub fn register_dimension_types(registry: &Registry<DimensionType>) {
    registry.register(
        DIMENSION_TYPE,
        DimensionType {
            fixed_time: None,
            sky_light: true,
            ceiling: false,
            ultra_warm: false,
            natural: true,
            coordinate_scale: 1.0,
            bed_works: false,
            respawn_anchor_works: false,
            min_height: -64,
            max_height: 384,
            max_logical_height: 384,
            infinite_burn_tag: InfiniteBurnTag::Overworld,
            effects: DimensionEffects::Overworld,
            ambient_light: 0.0,
            monster_settings: MonsterSettings {
                piglin_safe: false,
                raids: false,
                monster_spawn_light_level: 0,
                monster_spawn_block_light_limit: 0,
            },
        },
    );
}
