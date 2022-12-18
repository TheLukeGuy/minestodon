use crate::mc::net::pre_login::{Listing, ListingPlayers, ListingVersion};
use crate::mc::net::Connection;
use crate::mc::text::{HexTextColor, Text};
use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::thread;

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn bind(addr: &str) -> Result<Self> {
        let listener = TcpListener::bind(addr)
            .with_context(|| format!("failed to bind a new TCP listener to {addr}"))?;

        info!("Bound a new server to {addr}!");
        Ok(Self { listener })
    }

    pub fn run(self) {
        let rc = Arc::new(RwLock::new(self));
        ServerRef(rc).run();
    }
}

pub struct ServerRef(Arc<RwLock<Server>>);

impl ServerRef {
    pub fn run(&self) {
        loop {
            if let Err(err) = self.tick() {
                error!("Failed to tick the server:\nError: {err:?}");
            }
        }
    }

    fn tick(&self) -> Result<()> {
        let (stream, addr) = self
            .read_lock()
            .listener
            .accept()
            .context("failed to accept the incoming connection")?;
        debug!("Accepted a new connection from {addr}.");

        let clone = Self::clone(self);
        thread::Builder::new()
            .name(format!("user/{addr}"))
            .spawn(|| User::new(clone, stream).run())
            .context("failed to spawn a user thread")?;
        Ok(())
    }

    pub fn listing(&self) -> Listing {
        Listing {
            version: ListingVersion {
                value: 761,
                name: "Minestodon 1.19.3".into(),
            },
            players: ListingPlayers {
                current: 0,
                max: 1,
                sample: None,
            },
            motd: Text::from("Minestodon!")
                .color(HexTextColor("#6364ff"))
                .bolded(true),
            icon: None,
        }
    }

    pub fn legacy_listing(&self) -> Listing {
        self.listing()
    }

    fn read_lock(&self) -> RwLockReadGuard<Server> {
        self.0
            .read()
            .expect("failed to acquire the server lock with read access")
    }
}

impl Clone for ServerRef {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

pub struct User {
    pub server: ServerRef,
    pub connection: Connection,
}

impl User {
    pub fn new(server: ServerRef, stream: TcpStream) -> Self {
        Self {
            server,
            connection: Connection::new(stream),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.tick() {
                Err(err) => {
                    error!("Failed to tick the user:\nError: {err:?}");
                    if let Err(err) = self.connection.send_error_kick(err) {
                        warn!("Failed to kick the player after an error: {err:?}");
                    }
                    break;
                }
                Ok(ShouldClose::True) => break,
                _ => (),
            }
        }
        debug!("Closing the connection.");
    }

    fn tick(&mut self) -> Result<ShouldClose> {
        self.connection
            .tick(&self.server)
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
