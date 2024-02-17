use clap::{arg, ArgAction, Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use console::style;
use lazy_static::lazy_static;
use std::fmt::Write;
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
    version: Option<bool>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Apply configuration
    #[command(alias = "p")]
    Provision(ProvisionArgs),

    /// Output resolved configuration
    #[command(alias = "r")]
    Resolve(ResolveArgs),

    /// Configure global variables
    #[command(subcommand)]
    Var(VarSubcommand),

    /// Generate shell completions
    #[command(after_long_help = COMPLETION_EXAMPLES_HELP.as_str())]
    Completion(CompletionArgs),

    /// Plugin utilities
    #[command(subcommand)]
    Plugin(PluginSubcommand),
}

#[derive(Debug, Subcommand)]
pub enum PluginSubcommand {
    /// Link plugin at path
    #[command(after_long_help = LINK_HELP.as_str())]
    Link(LinkArgs),

    /// Scripting language plugin helpers
    Helper(HelperArgs),

    /// Generate the assets for releasing a plugin
    #[command(after_long_help = PACK_HELP.as_str())]
    Pack(PackArgs),
}

#[derive(Debug, Subcommand)]
pub enum VarSubcommand {
    /// Set a global variable
    #[command(after_long_help = VAR_SET_HELP.as_str())]
    Set(VarSetArgs),
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
            .fold(String::new(), |mut output, e| {
                let _ = writeln!(output, "  $ nk completion {} > {}",
                e.shell,
                e.path.display());
                output
            })
    );
    static ref LINK_HELP: String = format!(
        "{}\n{}\n{}",
        style("Examples:").underlined().bold(),
        "  $ nk plugin link ./plugin.yml",
        "  $ nk plugin link ~/Projects/nk-plugins/*/plugin.yml"
    );
    static ref PACK_HELP: String = format!(
        "{}\n{}\n\n{}\n{}",
        style("Examples:").underlined().bold(),
        [
            "  $ nk plugin pack \\",
            "    --owner 'ciiqr' \\",
            "    --repo 'nk-plugins' \\",
            "    --version 'v0.12.0' \\",
            "    --output 'assets' \\",
            "    ~/Projects/nk-plugins/*/plugin.yml",
        ].join("\n"),
        style("Notes:").underlined().bold(),
        [
            "  - The file names for assets are generated based on the `when:` conditions in your plugin.yml",
            "  - Any simple (`var == \"value\"`) conditions for system vars will contribute to the file name",
            "    - System vars include: distro, os, family, and arch",
            "  - ie. `when: [os == \"macos\", arch == \"aarch64\"]`, will produce `{plugin}-macos-aarch64.tar.gz`",
            "  - For now, anything more complicated will need to be packed manually"
        ].join("\n")
    );
    static ref VAR_SET_HELP: String = format!(
        "{}\n{}\n{}",
        style("Examples:").underlined().bold(),
        "  $ nk var set machine 'some-machine'",
        "  $ nk var set roles '[some, roles]'"
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
    /// Path to a plugin.yml file
    #[arg(value_name = "path", required = true)]
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum PluginLanguage {
    Bash,
}

#[derive(Debug, Args)]
pub struct HelperArgs {
    #[arg(value_name = "language")]
    pub language: PluginLanguage,
}

#[derive(Debug, Args)]
pub struct PackArgs {
    /// Github repo owner
    #[arg(long, value_name = "owner")]
    pub owner: String,
    /// Github repo name
    #[arg(long, value_name = "repo")]
    pub repo: String,
    /// Release version
    #[arg(long, value_name = "version")]
    pub version: String,
    /// Output directory for assets
    #[arg(long, value_name = "output", default_value = "./")]
    pub output: PathBuf,
    /// Path to a plugin.yml file
    #[arg(value_name = "path", required = true)]
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub struct VarSetArgs {
    /// Variable name
    #[arg(value_name = "name")]
    pub name: String,

    /// Variable value (as yaml)
    #[arg(value_name = "value")]
    pub value: String,
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
