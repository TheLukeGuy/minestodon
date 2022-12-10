use crate::mc::pre_login::{Listing, ListingPlayers, ListingVersion};
use crate::mc::text::Text;
use crate::mc::Connection;
use anyhow::{Context, Result};

pub mod mc;

pub struct User {
    pub mc: Connection,
}

impl User {
    pub fn tick(&mut self) -> Result<ShouldClose> {
        self.mc
            .tick(test_listing)
            .context("failed to tick the Minecraft connection")
    }
}

pub enum ShouldClose {
    False,
    True,
}

impl ShouldClose {
    pub fn is_true(&self) -> bool {
        matches!(self, Self::True)
    }
}

fn test_listing() -> Listing {
    Listing {
        version: ListingVersion {
            value: 760,
            name: String::from("Minestodon 1.19.2"),
        },
        players: ListingPlayers {
            current: 0,
            max: 1,
            sample: None,
        },
        motd: Text::String(String::from("Minestodon")),
        icon: None,
    }
}
