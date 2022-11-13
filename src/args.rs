use std::{path::PathBuf, str::FromStr};

use crate::{
    commands::{PluginArgs, PluginSubcommand, ProvisionArgs, ResolveArgs, ResolveOutputFormat},
    extensions::{PicoArgsExt, VecOsStringToStringExt},
};

pub struct Arguments {
    pub global: GlobalArguments,
    pub subcommand: Subcommand,
}

pub struct GlobalArguments {
    pub help: bool,
    pub version: bool,
    pub config: Option<PathBuf>,
}

pub enum Subcommand {
    Provision { args: ProvisionArgs },
    Resolve { args: ResolveArgs },
    Plugin { args: PluginArgs },
    Help,
    Version,
}

pub fn parse_args() -> Result<Arguments, Box<dyn std::error::Error>> {
    let mut pargs = pico_args::Arguments::from_env();

    let global = parse_global(&mut pargs);
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
            remaining.to_str_vec()?.join(" ")
        )
        .into());
    }

    Ok(Arguments { global, subcommand })
}

fn parse_global(pargs: &mut pico_args::Arguments) -> GlobalArguments {
    GlobalArguments {
        help: pargs.contains_any(["-h", "--help"]),
        version: pargs.contains_any(["-v", "--version"]),
        config: pargs
            .opt_value_from_fn(["-c", "--config"], PathBuf::from_str)
            .unwrap_or(None),
    }
}

fn parse_subcommand(
    pargs: &mut pico_args::Arguments,
) -> Result<Subcommand, Box<dyn std::error::Error>> {
    match pargs.subcommand()?.as_deref() {
        Some("p" | "provision") => Ok(Subcommand::Provision {
            args: ProvisionArgs {
                show_unchanged: pargs.contains_any("--show-unchanged"),
            },
        }),
        Some("r" | "resolve") => Ok(Subcommand::Resolve {
            args: ResolveArgs {
                // TODO: consider --render true|false instead?
                render: !pargs.contains_any("--no-render"),
                output: pargs
                    .opt_value_from_fn("--output", |format| match format {
                        "yaml" => Ok(ResolveOutputFormat::Yaml),
                        "json" => Ok(ResolveOutputFormat::Json),
                        format => Err(format!("invalid output format: {}", format)),
                    })?
                    .unwrap_or(ResolveOutputFormat::Yaml),
            },
        }),
        Some("plugin") => Ok(Subcommand::Plugin {
            args: PluginArgs {
                subcommand: (match pargs.free_from_str::<String>() {
                    Ok(_) => Ok(PluginSubcommand::Bash), // NOTE: just assume for now
                    Err(e) => Err(e),
                })?,
            },
        }),
        Some("h" | "help") => Ok(Subcommand::Help),
        Some("v" | "version") => Ok(Subcommand::Version),
        Some(input) => Err(format!("unknown subcommand: {}", input).into()),
        None => Ok(Subcommand::Help),
    }
}
