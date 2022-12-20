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
