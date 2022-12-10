use crate::mc::pre_login::{Listing, ListingPlayers, ListingVersion};
use crate::mc::text::Text;
use crate::mc::Connection;
use anyhow::{Context, Result};

pub mod mc;

pub struct User {
    pub mc: Connection,
}

impl User {
    pub fn tick(&mut self) -> Result<()> {
        let packets = self
            .mc
            .tick(test_listing)
            .context("failed to tick the Minecraft connection")?;
        for packet in packets {
            self.mc
                .handle_pre_play_packet(&packet, test_listing)
                .context("failed to handle a pre-play packet")?;
        }

        Ok(())
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
