//! Loon server binary entry point.

use clap::Parser;
use loon_server::config::{Cli, ServerConfig};
use loon_server::logging;
use loon_server::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    logging::init_from_cli(&cli)?;

    let config = ServerConfig::load(&cli)?;
    run(config, cli.force_scan).await?;
    Ok(())
}
