mod command;
mod handler;

use anyhow::Result;
use handler::{debian::{Debian, DebianPackage}, RepositoryHandler};

#[tokio::main]
async fn main() -> Result<()> {
    //let args: Cli = Cli::parse();

    let debian_handler: Debian = Debian::default();
    let index_data: Vec<String> = debian_handler.fetch_package_data().await?;
    let out: Vec<DebianPackage> = debian_handler.parse_packages(index_data[0].clone());

    for data in out {
        println!("{data:?}");
    }

    Ok(())
}
