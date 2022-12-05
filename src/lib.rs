use crate::mc::Connection;
use anyhow::{Context, Result};

pub mod mc;

pub struct User {
    pub mc: Connection,
}

impl User {
    pub fn tick(&mut self) -> Result<()> {
        self.mc
            .tick()
            .context("failed to tick the Minecraft connection")?;
        Ok(())
    }
}
