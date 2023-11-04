use crate::{
    args::ProvisionArgs,
    config::Config,
    eval::{DeclaredState, Evaluator},
    plugins::{load_plugins, Plugin, ProvisionInfo, ProvisionStateStatus},
    resolve::{resolve, ResolveOptions},
    root::{ensure_not_root, sudo_prompt},
    sort::sort_execution_sets,
    vars::get_global_vars,
};
use console::style;
use jsonschema::JSONSchema;
use textwrap::indent;

// TODO: wrap most errors in our own, more user friendly error
pub async fn provision(
    args: ProvisionArgs,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    // ensure not running as root
    ensure_not_root()?;

    // run sudo once (so the user can be prompted for their password if required, and any additional sudo's will hopefully be within the password timeout period)
    // TODO: if this is insufficient, may want to consider running sudo periodically during the provision (to refresh the timeout)
    sudo_prompt()?;

    // initialize global vars
    let global_vars = get_global_vars()?;

    // initialize evaluator
    let evaluator = Evaluator::new(global_vars.clone());

    // load plugins
    let plugins = load_plugins(&config, &evaluator).await?;

    // resolve state
    let resolved = resolve(
        &config,
        &global_vars,
        &evaluator,
        &plugins,
        &ResolveOptions { render: true },
    )?;

    // match each state to a plugin (group states by their matching plugin)
    let mut execution_sets =
        evaluator.match_states_to_plugins(&resolved.declarations, &plugins)?;

    // sort execution sets
    sort_execution_sets(&mut execution_sets);

    // validate
    validate(&execution_sets)?;

    // provision
    let provision_info = ProvisionInfo {
        sources: config.sources,
        vars: resolved.vars,
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
                                    println!(
                                        "{}",
                                        style(format!("x {}", o.description))
                                            .green()
                                    );
                                };

                                Ok(())
                            }
                            (ProvisionStateStatus::Success, true) => {
                                println!(
                                    "{}",
                                    style(format!("- {}", o.description))
                                        .color256(208)
                                );
                                Ok(())
                            }
                            (ProvisionStateStatus::Failed, _) => {
                                println!(
                                    "{}",
                                    style(format!("! {}", o.description)).red()
                                );
                                // TODO: can we get the terminal tab size?
                                println!(
                                    "{}",
                                    indent(o.output.as_str(), "    ")
                                );
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
        .any(|pr| pr.iter().any(|r| r.iter().any(Result::is_err)))
    {
        return Err("provisioning failed...")?;
    }

    Ok(())
}

fn validate(
    execution_sets: &[(Plugin, Vec<DeclaredState>)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut any_validation_errors = false;
    for (plugin, states) in execution_sets {
        let json_schema = serde_json::to_value(&plugin.definition.schema)?;

        match JSONSchema::compile(&json_schema) {
            Ok(schema) => {
                for state in states {
                    let json_state = serde_json::to_value(state.state.clone())?;
                    if let Err(errors) = schema.validate(&json_state) {
                        for error in errors {
                            let state_str =
                                match serde_json::to_string(&state.state)
                                    .map_err(|e| {
                                        format!("{e}: {:?}", state.state)
                                    }) {
                                    Ok(v) | Err(v) => v,
                                };
                            println!(
                                "{}",
                                style(format!(
                                    "{error}: validating {}: {} against plugin: {}",
                                    state.declaration, state_str, plugin.definition.name
                                ))
                                .red()
                            );
                            any_validation_errors = true;
                        }
                    };
                }

                Ok(())
            }
            Err(e) => Err(format!(
                "error parsing '{}' plugin's schema: {}",
                plugin.definition.name, e
            )),
        }?;
    }

    if any_validation_errors {
        Err("validation error".into())
    } else {
        Ok(())
    }
}
