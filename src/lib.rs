use crate::mc::status::{Listing, ListingPlayers, ListingVersion};
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
            .tick()
            .context("failed to tick the Minecraft connection")?;
        for packet in packets {
            let listing = || Listing {
                version: ListingVersion {
                    value: 760,
                    name: String::from("Minestodon 1.19.2"),
                },
                players: ListingPlayers {
                    current: 0,
                    max: 1,
                    sample: vec![],
                },
                motd: Default::default(),
                icon: "".to_string(),
            };
            self.mc
                .handle_pre_play_packet(&packet, listing, listing)
                .context("failed to handle a pre-play packet")?;
        }

        Ok(())
    }
}
