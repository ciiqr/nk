use clap::{arg, ArgAction, Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use console::style;
use lazy_static::lazy_static;
use std::{fmt, path::PathBuf};

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

    /// Generate shell completions
    #[command(after_long_help = COMPLETION_EXAMPLES_HELP.as_str())]
    Completion(CompletionArgs),
}

pub struct CompletionFile {
    pub shell: Shell,
    pub path: PathBuf,
}

lazy_static! {
    pub static ref COMPLETION_FILES: [CompletionFile; 4] = [
        CompletionFile {
            shell: Shell::Bash,
            path: "/usr/local/share/bash-completion/completions/nk".into()
        },
        CompletionFile {
            shell: Shell::Bash,
            path: "/opt/homebrew/share/bash-completion/completions/nk".into()
        },
        CompletionFile {
            shell: Shell::Zsh,
            path: "/usr/local/share/zsh/site-functions/_nk".into()
        },
        CompletionFile {
            shell: Shell::Zsh,
            path: "/opt/homebrew/share/zsh/site-functions/_nk".into()
        }
    ];
    static ref COMPLETION_EXAMPLES_HELP: String = format!(
        "{}\n{}",
        style("Examples:").underlined().bold(),
        COMPLETION_FILES
            .iter()
            .map(|e| format!(
                "  $ nk completion {} > {}\n",
                e.shell,
                e.path.display()
            ))
            .collect::<String>()
    );
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
    /// path do a plugin directory (containing a plugin.yml)
    #[arg(value_name = "path")]
    pub path: PathBuf,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum PluginLanguage {
    Bash,
}

#[derive(Debug, Args)]
pub struct PluginArgs {
    #[arg(value_name = "language")]
    pub language: PluginLanguage,
}

#[derive(Debug, Subcommand)]
pub enum CompletionCommand {
    /// Install completions
    Install,

    // NOTE: based on clap_complete::Shell
    /// Print Bourne Again SHell (bash) completions
    Bash,
    /// Print Elvish shell completions
    Elvish,
    /// Print Friendly Interactive SHell (fish) completions
    Fish,
    /// Print PowerShell completions
    #[command(name = "powershell")]
    PowerShell,
    /// Print Z SHell (zsh) completions
    Zsh,
}

impl CompletionCommand {
    pub const fn as_shell(&self) -> Option<Shell> {
        match self {
            CompletionCommand::Install => None,
            CompletionCommand::Bash => Some(Shell::Bash),
            CompletionCommand::Elvish => Some(Shell::Elvish),
            CompletionCommand::Fish => Some(Shell::Fish),
            CompletionCommand::PowerShell => Some(Shell::PowerShell),
            CompletionCommand::Zsh => Some(Shell::Zsh),
        }
    }
}

#[derive(Debug, Args)]
pub struct CompletionArgs {
    #[command(subcommand)]
    pub command: CompletionCommand,
}
