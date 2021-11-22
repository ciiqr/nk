use crate::extensions::{pico_args::PicoArgsExt, vec_os_string::VecOsStringToStringExt};

pub struct Arguments {
    pub global: GlobalArguments,
    pub subcommand: Subcommand,
}

pub struct GlobalArguments {
    pub help: bool,
    pub version: bool,
}

pub enum Subcommand {
    Provision { dry_run: bool },
    Help,
    Version,
}

pub fn parse_args() -> Result<Arguments, Box<dyn std::error::Error>> {
    let mut pargs = pico_args::Arguments::from_env();

    let global = GlobalArguments {
        help: pargs.contains_any(["-h", "--help"]),
        version: pargs.contains_any(["-v", "--version"]),
    };

    let provided_subcommand = parse_subcommand(&mut pargs)?;

    // NOTE: -h/-v override the provided subcommand
    // - we still parse it though so its arguments aren't unused
    let subcommand = if global.help {
        Subcommand::Help
    } else if global.version {
        Subcommand::Version
    } else {
        provided_subcommand
    };

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

fn parse_subcommand(
    pargs: &mut pico_args::Arguments,
) -> Result<Subcommand, Box<dyn std::error::Error>> {
    match pargs.subcommand()?.as_deref() {
        Some("p" | "provision") => Ok(Subcommand::Provision {
            dry_run: pargs.contains_any("--dry-run"),
        }),
        Some("h" | "help") => Ok(Subcommand::Help),
        Some("v" | "version") => Ok(Subcommand::Version),
        Some(input) => Err(format!("unknown subcommand: {}", input).into()),
        None => Ok(Subcommand::Help),
    }
}
