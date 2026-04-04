use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "nm-codegen")]
#[command(about = "NetworkManager binding code generator")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    FetchSpec {
        #[arg(long)]
        root_url: String,

        #[arg(long)]
        out_dir: PathBuf,
    },
    Generate {
        #[arg(long)]
        snapshot_dir: PathBuf,

        #[arg(long)]
        output: PathBuf,

        #[arg(long)]
        config: Option<PathBuf>,
    },
}
