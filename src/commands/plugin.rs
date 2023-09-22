static BASH_PLUGIN_UTILITIES: &str = include_str!("../plugins/utils/bash.sh");

#[derive(Debug)]
pub enum PluginSubcommand {
    Bash,
}

#[derive(Debug)]
pub struct PluginArgs {
    pub subcommand: PluginSubcommand,
}

pub fn plugin(args: &PluginArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.subcommand {
        PluginSubcommand::Bash => {
            println!("{}", BASH_PLUGIN_UTILITIES);
        }
    }

    Ok(())
}
