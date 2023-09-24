use crate::args::{PluginArgs, PluginLanguage};

static BASH_PLUGIN_UTILITIES: &str = include_str!("../plugins/utils/bash.sh");

pub fn plugin(args: &PluginArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.language {
        PluginLanguage::Bash => {
            println!("{}", BASH_PLUGIN_UTILITIES);
        }
    }

    Ok(())
}
