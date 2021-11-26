mod args;
mod commands;
mod extensions;
mod state;

use args::{parse_args, Subcommand};
use commands::{help, provision, version};
use std::process::exit;

fn main() {
    if let Err(err) = run() {
        eprintln!("nk: {}", err);
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = parse_args()?;

    match arguments.subcommand {
        Subcommand::Provision { args } => provision(args),
        Subcommand::Help => help(),
        Subcommand::Version => version(),
    }
}
