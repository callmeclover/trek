mod command;
mod handler;

use anyhow::Result;
use handler::{
    debian::Debian,
    RepositoryHandler,
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut debian_handler: Debian = Debian::default();
    debian_handler.sync_repository().await?;

    Ok(())
}
