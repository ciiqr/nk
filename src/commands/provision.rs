use crate::{
    config::Config,
    eval::Evaluator,
    merge::merge_groups,
    plugins::{Plugin, ProvisionStateStatus},
    state,
};
use console::style;
use std::collections::HashMap;

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
    // println!("plugins: {:#?}", plugins);

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
    // println!("TODO: implement provision:");
    // println!("{:?}", args);
    // println!("{:#?}", config);
    // println!("{:#?}", machine);
    // println!("files: {:#?}", files);
    // println!("groups: {:#?}", groups);
    // println!("resolved: {:#?}", resolved);
    // println!("execution_sets: {:#?}", execution_sets);

    // TODO: decide how I'm going to provide helpers to plugins
    // TODO: maybe just download jq (and possibly other utilities) and inject it into the path before running plugins...

    // // bootstrap
    // for (plugin, values) in &execution_sets {
    //     // TODO: handle errors better
    //     plugin.bootstrap()?;
    // }

    // provision
    let provision_results = execution_sets
        .iter()
        .map(|(p, v)| match p.provision(&args, v) {
            Ok(i) => {
                Ok(i.map(|r| match r {
                    Ok(o) => {
                        // provisioning a single result
                        // TODO: format: "[x!] {plugin}: {description}"
                        // TODO: include "output" indented for failed results
                        match (o.status, o.changed) {
                            (ProvisionStateStatus::Success, false) => {
                                // TODO: make this a cli flag
                                let show_unchanged = true;
                                if show_unchanged {
                                    println!("{}", style(format!("x {}", o.description)).green());
                                };

                                Ok(())
                            }
                            (ProvisionStateStatus::Success, true) => {
                                println!("{}", style(format!("x {}", o.description)).green());
                                Ok(())
                            }
                            (ProvisionStateStatus::Failed, _) => {
                                println!("{}", style(format!("! {}", o.description)).red());
                                Err("provisioning failed".to_string()) // TODO: idk about this message...
                            }
                        }
                    }
                    Err(e) => {
                        // provisioning a single result failed, ie. maybe just output parsing error
                        // TODO: decide format...
                        println!("{}", e);
                        Err(e.to_string())
                    }
                })
                .collect::<Vec<Result<_, _>>>())
            }
            Err(e) => {
                // provisioning as a whole failed for this plugin
                // TODO: decide format...
                println!("{}", e);
                Err(e)
                // Ok()
                // vec![Err(e)]
            }
        })
        .collect::<Result<Vec<_>, _>>();

    // TODO: list unmatched states

    // TODO: ugh...
    if provision_results
        .iter()
        .any(|pr| pr.iter().any(|r| r.iter().any(|r| r.is_err())))
    {
        Err("provisioning failed...")?
    }

    Ok(())
}
