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

    // TODO: match states to plugins

    // TODO: run plugins:
    for config_plugin in &config.plugins {
        let plugin = Plugin::from_config(config_plugin)?;
        println!("plugin_definition: {:?}", plugin.definition);

        // TODO: handle errors better
        plugin.initialize()?;

        // TODO: per-state?
        // plugin.match(state)?;

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
