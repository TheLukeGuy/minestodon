use crate::mc::world::{Biome, DimensionType};
use crate::mc::{world, Identifier};
use minestodon_macros::minecraft;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::sync::RwLock;

pub static BIOMES: Registry<Biome> =
    Registry::new(minecraft!("worldgen/biome"), world::register_biomes);

pub static DIMENSION_TYPES: Registry<DimensionType> = Registry::new(
    minecraft!("dimension_type"),
    world::register_dimension_types,
);

// TODO: Actually implement message types
pub static MESSAGE_TYPES: Registry<()> = Registry::new(minecraft!("chat_type"), |_| {});

pub fn init() {
    BIOMES.init();
    DIMENSION_TYPES.init();
    MESSAGE_TYPES.init();
}

pub struct Registry<T> {
    pub id: Identifier,
    entries: RwLock<Option<HashMap<Identifier, T>>>,
    init_fn: fn(&Registry<T>),
}

impl<T> Registry<T> {
    pub const fn new(id: Identifier, init: fn(&Registry<T>)) -> Self {
        Self {
            id,
            entries: RwLock::new(None),
            init_fn: init,
        }
    }

    pub fn init(&self) {
        {
            let mut entries = self.entries.write().unwrap();
            if entries.is_some() {
                panic!("the registry has already been initialized");
            }
            *entries = Some(HashMap::new());
        }
        (self.init_fn)(self);
    }

    pub fn register(&self, key: Identifier, value: T) {
        self.write_entries(|entries| {
            if entries.contains_key(&key) {
                panic!("the registry already contains {key}");
            }
            entries.insert(key, value);
        });
    }

    fn read_entries<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<Identifier, T>) -> R,
    {
        let locked = self.entries.read().unwrap();
        let entries = locked
            .as_ref()
            .expect("the registry is not yet initialized");
        f(entries)
    }

    fn write_entries<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<Identifier, T>) -> R,
    {
        let mut locked = self.entries.write().unwrap();
        let entries = locked
            .as_mut()
            .expect("the registry is not yet initialized");
        f(entries)
    }
}

#[derive(Serialize)]
struct SerializableRegistryEntry<'a, T>
where
    T: Serialize,
{
    pub name: &'a Identifier,
    pub id: i32,
    pub element: &'a T,
}

impl<T> Serialize for Registry<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.read_entries(|entries| {
            let entries = entries
                .iter()
                .enumerate()
                .filter_map(|(id, (name, element))| {
                    id.try_into()
                        .ok()
                        .map(|id| SerializableRegistryEntry { name, id, element })
                })
                .collect::<Vec<_>>();

            let mut map = serializer.serialize_map(Some(2))?;
            map.serialize_entry("type", &self.id)?;
            map.serialize_entry("value", &entries)?;
            map.end()
        })
    }
}
