use anyhow::{bail, Result};
use serde::{Serialize, Serializer};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::{fmt, result};

pub mod entity;
pub mod net;
pub mod text;
pub mod world;

pub struct Identifier {
    namespace: Cow<'static, str>,
    // TODO: Use a `Cow<'static, str>` for the path
    path: String,
}

impl Identifier {
    const MINECRAFT: &'static str = "minecraft";

    pub fn try_new(namespace: impl Into<Cow<'static, str>>, path: &str) -> Result<Self> {
        let namespace = namespace.into();
        let path = path.into();

        let valid = |c: char| matches!(c, 'a'..='z' | '0'..='9' | '.' | '-' | '_');
        if !namespace.chars().all(valid) {
            bail!("the namespace contains invalid characters");
        }
        if !namespace.chars().all(|c| valid(c) || c == '/') {
            bail!("the path contains invalid characters");
        }

        Ok(Self { namespace, path })
    }

    pub fn new(namespace: impl Into<Cow<'static, str>>, path: &str) -> Self {
        Self::try_new(namespace, path).unwrap_or_else(|err| panic!("{err}"))
    }

    pub fn try_minecraft(path: &str) -> Result<Self> {
        Self::try_new(Self::MINECRAFT, path)
    }

    pub fn minecraft(path: &str) -> Self {
        Self::new(Self::MINECRAFT, path)
    }

    pub fn parse(str: &str) -> Result<Self> {
        if let Some((namespace, path)) = str.split_once(':') {
            Self::try_new(namespace.to_string(), path)
        } else {
            Self::try_new(Self::MINECRAFT, str)
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
