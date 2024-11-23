use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Add command
    Add(AddArgs),
    /// Sub command
    Sub(SubArgs),
}

#[derive(Args, Debug)]
pub struct AddArgs {
    #[arg(short, long)]
    name: Option<String>,
}

#[derive(Args, Debug)]
pub struct SubArgs {
    name: Option<String>,
}
