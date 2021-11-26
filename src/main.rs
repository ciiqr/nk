mod args;
mod commands;
mod extensions;
mod state;

use args::{parse_args, Subcommand};
use commands::{help, provision, version};

fn main() {
    if let Err(err) = run() {
        exit(1, &err);
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

fn exit(code: i32, err: &dyn std::fmt::Display) -> ! {
    eprintln!("nk: {}", err);
    std::process::exit(code);
}
