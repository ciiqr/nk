use crate::{
    config::Config,
    eval::Evaluator,
    merge::{merge_groups, merge_vars},
    plugins::{Plugin, ProvisionInfo, ProvisionStateStatus},
    render::render_group,
    state::{self, ResolvedGroup},
    vars::get_builtin_vars,
};
use console::style;
use textwrap::indent;

#[derive(Debug)]
pub struct ProvisionArgs {
    pub dry_run: bool,
    pub show_unchanged: bool,
}

// TODO: wrap most errors in our own, more user friendly error
pub fn provision(args: ProvisionArgs, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // determine machine/role information
    let machine = state::Machine::get_current(&config)?;
    let roles = state::Role::find_by_names(&machine.roles, &config.sources);

    // initialize builtin vars
    let builtin_vars = get_builtin_vars(&machine)?;

    // initialize evaluator (machine, roles, platform, etc.)
    let evaluator = Evaluator::new(builtin_vars.clone());

    // load plugins
    let all_plugins = config
        .plugins
        .iter()
        .map(Plugin::from_config)
        .collect::<Result<Vec<_>, _>>()?;

    // filter plugins for os
    let plugins = evaluator.filter_plugins(all_plugins)?;

    // find all state files for this machine
    let files = state::File::find_all(&config.sources, &roles)?;

    // filter groups based on conditions
    let groups = evaluator.filter_files_to_matching_groups(&files)?;

    // merge all groups into into single resolved state
    let resolved = groups.into_iter().fold(ResolvedGroup::new(), merge_groups);

    // render resolved
    let rendered = render_group(builtin_vars.clone(), resolved)?;

    // match each state to a plugin (group states by their matching plugin)
    let execution_sets = evaluator.match_states_to_plugins(&rendered.declarations, plugins)?;

    // provision
    let provision_info = ProvisionInfo {
        sources: config.sources,
        vars: merge_vars(builtin_vars, rendered.vars)?,
    };
    let provision_results = execution_sets
        .iter()
        .map(|(p, v)| match p.provision(&provision_info, v) {
            Ok(i) => {
                Ok(i.map(|r| match r {
                    Ok(o) => {
                        // provisioning a single result
                        // TODO: format: "[x!] {plugin}: {description}"
                        // TODO: include "output" indented for failed results
                        match (o.status, o.changed) {
                            (ProvisionStateStatus::Success, false) => {
                                if args.show_unchanged {
                                    println!("{}", style(format!("x {}", o.description)).green());
                                };

                                Ok(())
                            }
                            (ProvisionStateStatus::Success, true) => {
                                // TODO: changed/unchanged should probably have different prefix but same colour?
                                println!("{}", style(format!("x {}", o.description)).green());
                                Ok(())
                            }
                            (ProvisionStateStatus::Failed, _) => {
                                println!("{}", style(format!("! {}", o.description)).red());
                                // TODO: can we get the terminal tab size?
                                println!("{}", indent(o.output.as_str(), "    "));
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
