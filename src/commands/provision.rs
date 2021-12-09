use crate::{config::Config, plugins::Plugin, state};

#[derive(Debug)]
pub struct ProvisionArgs {
    pub dry_run: bool,
}

// TODO: wrap most errors in our own, more user friendly error
pub fn provision(args: ProvisionArgs, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // find all state files for this machine
    let machine = state::Machine::get_current(&config)?;
    let roles = state::Role::find_by_names(&machine.roles, &config.sources);
    let files = state::File::find_by_roles(&roles)?;

    // TODO: initialize base vars (machine, roles, ?sources)

    // TODO: filter based on "when:" conditions (files[].groups[].when)
    // TODO: load plugins
    // TODO: call setup command on all plugins to determine how to interface with them? maybe only once required?
    // TODO: maybe move plugin config to sources... likely with "when:" conditions OR:
    // TODO: maybe NEED a "plugin.yml" to add basic "when:" conditions

    // TODO: match states to plugins

    // TODO: run plugins:
    for config_plugin in &config.plugins {
        let plugin = Plugin::from_config(config_plugin)?;
        println!("plugin_definition: {:?}", plugin.definition);

        // TODO: handle errors better
        plugin.setup()?;
        plugin.provision()?;
    }

    // TODO: change this to use a propper logger
    println!("TODO: implement provision:");
    println!("{:?}", args);
    println!("{:#?}", config);
    println!("{:?}", machine);
    println!("{:#?}", roles);
    for file in files {
        println!("{:#?}", file);
    }

    Ok(())
}
