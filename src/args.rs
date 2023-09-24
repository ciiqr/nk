use std::{fmt, path::PathBuf};

use clap::{arg, ArgAction, Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(about, long_about = None, disable_version_flag = true, version)]
pub struct Arguments {
    /// Override the config file.
    #[arg(short, long, value_name = "file")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,

    // Print version
    #[arg(
        short = 'v',
        short_alias = 'V',
        long,
        action = clap::builder::ArgAction::Version,
    )]
    version: (),
}

#[derive(Subcommand)]
pub enum Commands {
    /// Apply configuration
    #[command(alias = "p")]
    Provision(ProvisionArgs),

    /// Output resolved configuration
    #[command(alias = "r")]
    Resolve(ResolveArgs),

    /// Link plugin at path
    Link(LinkArgs),

    /// Scripting language plugin helpers
    Plugin(PluginArgs),
}

#[derive(Debug, Args)]
pub struct ProvisionArgs {
    /// Whether to print unchanged results.
    #[arg(short, long)]
    pub show_unchanged: bool,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ResolveOutputFormat {
    Yaml,
    Json,
}

impl fmt::Display for ResolveOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResolveOutputFormat::Yaml => write!(f, "yaml"),
            ResolveOutputFormat::Json => write!(f, "json"),
        }
    }
}

#[derive(Debug, Args)]
pub struct ResolveArgs {
    #[arg(short, long, value_name = "format", default_value_t = ResolveOutputFormat::Yaml)]
    pub output: ResolveOutputFormat,

    /// Don't replace templated values
    #[arg(long = "no-render", default_value_t = true, action = ArgAction::SetFalse)]
    pub render: bool,
}

#[derive(Debug, Args)]
pub struct LinkArgs {
    #[arg(value_name = "path")]
    pub path: PathBuf,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum PluginLanguage {
    Bash,
}

#[derive(Debug, Args)]
pub struct PluginArgs {
    pub language: PluginLanguage,
}
