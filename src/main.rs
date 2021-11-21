mod args;
mod commands;
mod extensions;

use args::{parse_args, Subcommand};
use commands::provision::provision;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const VERSION: &str = const_format::formatcp!("nk version {}", CARGO_PKG_VERSION);
const USAGE: &str = indoc::indoc! {"
    usage: nk [-h|--help] [-v|--version] <command> [<args>...]
      nk p|provision [--dry-run]
      nk h|help
      nk v|version
"};

fn main() {
    match parse_args() {
        Ok(args) => {
            if args.global.help {
                return help();
            }

            if args.global.version {
                return version();
            }

            match args.subcommand {
                Subcommand::Provision { dry_run } => provision(dry_run),
                Subcommand::Help => help(),
                Subcommand::Version => version(),
            }
        }
        Err(err) => exit(1, &err),
    }
}

fn exit(code: i32, err: &dyn std::fmt::Display) -> ! {
    eprintln!("nk: {}", err);
    std::process::exit(code);
}

fn help() {
    println!("{}", USAGE);
}

fn version() {
    println!("{}", VERSION);
}
