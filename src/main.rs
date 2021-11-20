use crate::extensions::{pico_args::PicoArgsExt, vec_os_string::VecOsStringToStringExt};
use std::{error::Error, fmt::Debug};

mod extensions;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const VERSION: &str = const_format::formatcp!("nk version {}", CARGO_PKG_VERSION);
const USAGE: &str = indoc::indoc! {"
    usage: nk [-h|--help] [-v|--version] <command> [<args>...]
      nk p|provision [--dry-run]
      nk h|help
      nk v|version
"};

fn help() {
    println!("{}", USAGE);
}

fn version() {
    println!("{}", VERSION);
}

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
                Subcommand::Provision { dry_run } => {
                    println!("TODO: run provision subcommand with dry_run={}", dry_run)
                }
                Subcommand::Help => {
                    help();
                }
                Subcommand::Version => {
                    version();
                }
            }
        }
        Err(err) => {
            eprintln!("nk: {}", err);
            std::process::exit(1);
        }
    }
}

// TODO: move
#[derive(Debug)]
struct Arguments {
    global: GlobalArguments,
    subcommand: Subcommand,
}

#[derive(Debug)]
struct GlobalArguments {
    help: bool,
    version: bool,
}

#[derive(Debug)]
enum Subcommand {
    Provision { dry_run: bool },
    Help,
    Version,
}

fn parse_args() -> Result<Arguments, Box<dyn Error>> {
    let mut pargs = pico_args::Arguments::from_env();

    let global = GlobalArguments {
        help: pargs.contains_any(["-h", "--help"]),
        version: pargs.contains_any(["-v", "--version"]),
    };

    let res: Result<Subcommand, String> = match pargs.subcommand()?.as_deref() {
        Some("p" | "provision") => Ok(Subcommand::Provision {
            dry_run: pargs.contains_any("--dry-run"),
        }),
        Some("h" | "help") => Ok(Subcommand::Help),
        Some("v" | "version") => Ok(Subcommand::Version),
        Some(input) => Err(format!("unknown subcommand: {}", input).into()),
        None => Ok(Subcommand::Help),
    };
    // TODO: this is kind of odd, with the second let... need a better way to define error typw
    let subcommand = res?;

    let remaining = pargs.finish();
    if !remaining.is_empty() {
        return Err(format!(
            "unrecognized arguments: {}",
            remaining.to_str_vec().join(" ")
        )
        .into());
    }

    return Ok(Arguments { global, subcommand });
}
