mod args;
mod commands;
mod extensions;
mod state;

use args::{parse_args, Subcommand};
use commands::{help, provision, version};

fn main() {
    match parse_args() {
        Ok(args) => match match args.subcommand {
            Subcommand::Provision { args } => provision(args),
            Subcommand::Help => help(),
            Subcommand::Version => version(),
        } {
            Ok(_) => (),
            Err(err) => exit(1, &err),
        },
        Err(err) => exit(1, &err),
    }
}

fn exit(code: i32, err: &dyn std::fmt::Display) -> ! {
    eprintln!("nk: {}", err);
    std::process::exit(code);
}
