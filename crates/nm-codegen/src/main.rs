use anyhow::{Context, Result};
use clap::Parser;
use std::fs;

use nm_codegen::{
    RenderConfig,
    cli::{Args, Command},
    fetch::spec::fetch_spec_snapshot,
    generate_types_from_snapshot_dir, load_render_config,
};

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::FetchSpec { root_url, out_dir } => {
            fetch_spec_snapshot(&root_url, &out_dir)
                .with_context(|| format!("failed to fetch spec from {root_url}"))?;
        }
        Command::Generate {
            snapshot_dir,
            output,
            config,
        } => {
            let config = match config {
                Some(path) => load_render_config(&path)?,
                None => RenderConfig::default(),
            };

            let generated =
                generate_types_from_snapshot_dir(&snapshot_dir, &config).with_context(|| {
                    format!(
                        "failed to generate types from snapshot {}",
                        snapshot_dir.display()
                    )
                })?;

            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }

            fs::write(&output, generated)
                .with_context(|| format!("failed to write to {}", output.display()))?;
        }
    }

    Ok(())
}
