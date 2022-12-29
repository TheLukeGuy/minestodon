use anyhow::{bail, Result};
use serde::{Serialize, Serializer};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::{fmt, result};

pub mod net;
pub mod player;
pub mod registry;
pub mod text;
pub mod world;

#[derive(Eq, PartialEq, Hash)]
pub struct Identifier {
    namespace: Cow<'static, str>,
    path: Cow<'static, str>,
}

impl Identifier {
    pub const MINECRAFT: &'static str = "minecraft";
    pub const MINESTODON: &'static str = "minestodon";

    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Result<Self> {
        let namespace = namespace.into();
        let path = path.into();

        let valid = |c| matches!(c, 'a'..='z' | '0'..='9' | '.' | '-' | '_');
        if !namespace.chars().all(valid) {
            bail!("the namespace contains invalid characters");
        }
        if !namespace.chars().all(|c| valid(c) || c == '/') {
            bail!("the path contains invalid characters");
        }

        let identifier = Self {
            namespace: namespace.into(),
            path: path.into(),
        };
        Ok(identifier)
    }

    /// # Safety
    ///
    /// The given path must contain only lowercase alphanumeric characters, dots (`.`),
    /// dashes (`-`), underscores (`_`), and slashes (`/`).
    pub const unsafe fn new_unchecked(namespace: &'static str, path: &'static str) -> Self {
        Self {
            namespace: Cow::Borrowed(namespace),
            path: Cow::Borrowed(path),
        }
    }

    pub fn parse(str: &str) -> Result<Self> {
        if let Some((namespace, path)) = str.split_once(':') {
            Self::new(namespace.to_string(), path)
        } else {
            Self::new(Self::MINECRAFT, str)
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
