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
use args::{PluginSubcommand, VarSubcommand};
use clap::CommandFactory;
use clap::Parser;
use commands::{completion, helper, link, pack, provision, resolve, var_set};
use config::Config;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = run().await {
        eprintln!("nk: {}", err);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Arguments::command();
    let arguments = Arguments::parse();
    let config = Config::new(&arguments);

    match arguments.command {
        Some(Commands::Provision(args)) => provision(args, config?).await,
        Some(Commands::Resolve(args)) => resolve(args, config?).await,
        Some(Commands::Completion(args)) => completion(&args, &mut cmd),
        Some(Commands::Var(subcommand)) => match subcommand {
            VarSubcommand::Set(args) => var_set(args),
        },
        Some(Commands::Plugin(subcommand)) => match subcommand {
            PluginSubcommand::Link(args) => link(&args),
            PluginSubcommand::Helper(args) => helper(&args),
            PluginSubcommand::Pack(args) => pack(args),
        },
        None => {
            cmd.print_help()?;
            Ok(())
        }
    }
}
