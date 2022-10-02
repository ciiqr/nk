use crate::{config::Config, eval::Evaluator, plugins::Plugin, state};

#[derive(Debug)]
pub struct ProvisionArgs {
    pub dry_run: bool,
}

// TODO: wrap most errors in our own, more user friendly error
pub fn provision(args: ProvisionArgs, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // load plugins
    let plugins = config
        .plugins
        .iter()
        .map(Plugin::from_config)
        .collect::<Result<Vec<_>, _>>()?;

    // determine machine/role information
    let machine = state::Machine::get_current(&config)?;
    let roles = state::Role::find_by_names(&machine.roles, &config.sources);

    // initialize base vars & evaluator (machine, roles, platform, etc.)
    let evaluator = Evaluator::new(&machine);

    // find all state files for this machine
    let files = state::File::find_all(&config.sources, &roles)?;

    // filter groups based on conditions
    let groups = evaluator.filter_files_to_matching_groups(&files)?;

    // TODO: merge all filtered files into into single resolved state

    // TODO: ? once we need it, apply custom vars from resolved state

    // TODO: match each state to a plugin (group states by their matching plugin)
    // plugin.match(state)?;

    // TODO: change this to use a propper logger
    println!("TODO: implement provision:");
    println!("{:?}", args);
    println!("{:#?}", config);
    println!("plugins: {:#?}", plugins);
    println!("{:#?}", machine);
    println!("files: {:#?}", files);
    println!("groups: {:#?}", groups);

    // bootstrap
    for plugin in &plugins {
        // TODO: only bootstrap plugins that matched
        // TODO: handle errors better
        plugin.bootstrap()?;
    }

    // provision
    for plugin in &plugins {
        // TODO: only provision plugins that matched (with the states that matched)
        // TODO: keep all results & partition at the end
        plugin.provision()?;
    }

    // TODO: check provisioning status of all states, exit based on results

    Ok(())
}
