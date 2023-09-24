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

use args::{Arguments, Commands};
use clap::CommandFactory;
use clap::Parser;
use commands::{link, plugin, provision, resolve};
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
    let arguments = Arguments::parse();
    let config = Config::new(&arguments);

    match arguments.command {
        Some(Commands::Provision(args)) => provision(args, config?).await,
        Some(Commands::Resolve(args)) => resolve(args, config?).await,
        Some(Commands::Link(args)) => link(&args),
        Some(Commands::Plugin(args)) => plugin(&args),
        None => {
            let mut cmd = Arguments::command();
            cmd.print_help()?;
            Ok(())
        }
    }
}
