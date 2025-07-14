use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cleansh::Cli::parse();
    cleansh::run(cli)
}