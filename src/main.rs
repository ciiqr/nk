mod args;
mod commands;
mod config;
mod eval;
mod extensions;
mod merge;
mod plugins;
mod state;
mod traits;
mod utils;

use args::{parse_args, Subcommand};
use commands::{help, provision, version};
use config::Config;
use std::process::exit;

fn main() {
    if let Err(err) = run() {
        eprintln!("nk: {}", err);
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = parse_args()?;
    let config = Config::new(&arguments);

    match arguments.subcommand {
        Subcommand::Provision { args } => provision(args, config?),
        Subcommand::Help => help(),
        Subcommand::Version => version(),
    }
}

// TODO: make sure not run as root
// TODO: figure out how best to support sudo with plugins... (worse case could probably have them provide some plugin definition field that indicates they need to run as root)
