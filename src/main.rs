use anyhow::{Context, Result};
use log::LevelFilter;
use minestodon::mc::registry;
use minestodon::server::Server;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode, ThreadLogMode};

fn main() -> Result<()> {
    init_logging().context("failed to initialize logging")?;
    registry::init();

    Server::bind("0.0.0.0:25565")
        .context("failed to create and bind the server")?
        .run();
    Ok(())
}

fn init_logging() -> Result<()> {
    TermLogger::init(
        LevelFilter::Trace,
        ConfigBuilder::new()
            .set_thread_level(LevelFilter::Error)
            .set_target_level(LevelFilter::Off)
            .set_thread_mode(ThreadLogMode::Both)
            .set_time_offset_to_local()
            .unwrap_or_else(|err| err)
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;
    Ok(())
}
