use std::collections::HashMap;

use crate::{config::Config, eval::Evaluator, merge::merge_groups, plugins::Plugin, state};

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
    println!("plugins: {:#?}", plugins);

    // determine machine/role information
    let machine = state::Machine::get_current(&config)?;
    let roles = state::Role::find_by_names(&machine.roles, &config.sources);

    // initialize base vars & evaluator (machine, roles, platform, etc.)
    let evaluator = Evaluator::new(&machine);

    // find all state files for this machine
    let files = state::File::find_all(&config.sources, &roles)?;

    // filter groups based on conditions
    let groups = evaluator.filter_files_to_matching_groups(&files)?;

    // merge all groups into into single resolved state
    let resolved = groups.iter().fold(
        state::Group {
            when: vec![],
            declarations: HashMap::new(),
        },
        merge_groups,
    );

    // TODO: ? once we need it, apply custom vars from resolved state

    // match each state to a plugin (group states by their matching plugin)
    let execution_sets = evaluator.match_states_to_plugins(&resolved.declarations, plugins)?;

    // TODO: change this to use a propper logger
    println!("TODO: implement provision:");
    println!("{:?}", args);
    println!("{:#?}", config);
    println!("{:#?}", machine);
    println!("files: {:#?}", files);
    println!("groups: {:#?}", groups);
    println!("resolved: {:#?}", resolved);
    println!("execution_sets: {:#?}", execution_sets);

    // // bootstrap
    // for (plugin, values) in &execution_sets {
    //     // TODO: handle errors better
    //     plugin.bootstrap()?;
    // }

    // provision
    for (plugin, values) in &execution_sets {
        // TODO: keep all results & partition at the end
        plugin.provision()?;
    }

    // TODO: check provisioning status of all states, exit based on results

    Ok(())
}
