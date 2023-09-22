#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(
    clippy::unnecessary_wraps,
    clippy::multiple_crate_versions,
    clippy::uninlined_format_args,
    clippy::module_name_repetitions,
    clippy::use_self,
    clippy::too_many_lines,
    // TODO: fix:
    clippy::future_not_send
)]

mod args;
mod commands;
mod config;
mod eval;
mod extensions;
mod merge;
mod plugins;
mod render;
mod resolve;
mod root;
mod sort;
mod state;
mod traits;
mod utils;
mod vars;

use args::{parse_args, Subcommand};
use commands::{help, link, plugin, provision, resolve, version};
use config::Config;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = run().await {
        eprintln!("nk: {}", err);
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = parse_args()?;
    let config = Config::new(&arguments);

    match arguments.subcommand {
        Subcommand::Provision { args } => provision(args, config?).await,
        Subcommand::Resolve { args } => resolve(args, config?).await,
        Subcommand::Link { args } => link(&args),
        Subcommand::Plugin { args } => plugin(&args),
        Subcommand::Help => help(),
        Subcommand::Version => version(),
    }
}
