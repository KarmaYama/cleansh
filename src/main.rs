// cleansh/src/main.rs
// Main entry point for the cleansh application.

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cleansh::Cli::parse();
    cleansh::run(cli)
}