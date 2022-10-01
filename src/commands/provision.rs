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
    let files = state::File::find_all(&config.sources, &roles)?;

    // TODO: initialize base vars (machine, roles, platform, etc.)

    // TODO: filter files based on "when:" conditions (files[].groups[].when)

    // load plugins
    let plugins = config
        .plugins
        .iter()
        .map(Plugin::from_config)
        .collect::<Result<Vec<_>, _>>()?;

    // TODO: match each state to a plugin (group states by their matching plugin)
    // plugin.match(state)?;

    // TODO: change this to use a propper logger
    println!("TODO: implement provision:");
    println!("{:?}", args);
    println!("{:#?}", config);
    println!("{:#?}", machine);
    println!("files: {:#?}", files);
    println!("plugins: {:#?}", plugins);

    // TODO: only bootstrap plugins that matched
    // bootstrap
    for plugin in &plugins {
        // TODO: handle errors better
        plugin.bootstrap()?;
    }

    // TODO: only provision plugins that matched (with the states that matched)
    // provision
    for plugin in &plugins {
        // TODO: handle errors better
        plugin.provision()?;
    }

    Ok(())
}
