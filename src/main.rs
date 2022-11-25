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
use commands::{help, plugin, provision, resolve, version};
use config::Config;
use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(err) = run() {
        eprintln!("nk: {}", err);
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = parse_args()?;
    let config = Config::new(&arguments);

    match arguments.subcommand {
        Subcommand::Provision { args } => provision(args, config?),
        Subcommand::Resolve { args } => resolve(args, config?),
        Subcommand::Plugin { args } => plugin(args),
        Subcommand::Help => help(),
        Subcommand::Version => version(),
    }
}
