use std::io::{stdout, Write};

use crate::{
    args::{ProvisionArgs, ProvisionOutputFormat},
    config::Config,
    eval::{DeclaredState, Evaluator},
    plugins::{
        load_plugins, Plugin, ProvisionInfo, ProvisionStateOutput,
        ProvisionStateStatus,
    },
    resolve::{resolve, ResolveOptions},
    root::{ensure_not_root, sudo_prompt},
    sort::sort_execution_sets,
    vars::get_global_vars,
};
use console::style;
use itertools::Itertools;
use jsonschema::Validator;
use textwrap::indent;

trait Formatter {
    // TODO: refactor to accept an iterator? (may allow more advanced formatting...)
    fn write_result(
        &self,
        writer: &mut dyn Write,
        // TODO: generic context object?
        args: &ProvisionArgs,
        res: &Result<ProvisionStateOutput, serde_json::Error>,
        raw: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

struct RawFormatter {}
impl Formatter for RawFormatter {
    fn write_result(
        &self,
        writer: &mut dyn Write,
        _args: &ProvisionArgs,
        _res: &Result<ProvisionStateOutput, serde_json::Error>,
        raw: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(writer, "{}", raw)?;
        Ok(())
    }
}

struct PrettyFormatter {}
impl Formatter for PrettyFormatter {
    fn write_result(
        &self,
        writer: &mut dyn Write,
        args: &ProvisionArgs,
        res: &Result<ProvisionStateOutput, serde_json::Error>,
        raw: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match res {
            Ok(o) => {
                // TODO: consider plugin context: "[x!] {plugin}: {description}"
                match (&o.status, o.changed) {
                    (ProvisionStateStatus::Success, false) => {
                        if args.show_unchanged {
                            writeln!(
                                writer,
                                "{}",
                                style(format!("x {}", o.description)).green()
                            )?;
                        };
                    }
                    (ProvisionStateStatus::Success, true) => {
                        writeln!(
                            writer,
                            "{}",
                            style(format!("- {}", o.description)).color256(208)
                        )?;
                    }
                    (ProvisionStateStatus::Failed, _) => {
                        writeln!(
                            writer,
                            "{}",
                            style(format!("! {}", o.description)).red()
                        )?;
                        // TODO: can we get the terminal tab size?
                        writeln!(
                            writer,
                            "{}",
                            indent(o.output.as_str(), "    ")
                        )?;
                    }
                }
            }
            Err(e) => {
                // provisioning a single result failed, likely parsing error from extraneous output
                // TODO: need plugin context so we can print the plugin that produced the error
                writeln!(
                    writer,
                    "{}",
                    style(format!("!! {} received: {}", e, raw))
                        .red()
                        .underlined()
                )?;
            }
        }

        Ok(())
    }
}

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

    let formatter: Box<dyn Formatter> = match args.output {
        ProvisionOutputFormat::Pretty => Box::new(PrettyFormatter {}),
        ProvisionOutputFormat::Raw => Box::new(RawFormatter {}),
    };

    // provision
    let provision_info = ProvisionInfo {
        sources: config.sources,
        vars: resolved.vars,
    };
    let mut lock = stdout().lock();
    let writer = lock.by_ref();
    let provision_results = execution_sets
        .iter()
        .map(|(p, v)| {
            match p.provision(&provision_info, v) {
                Ok(i) => Ok(i
                    .flatten()
                    .map(|line| {
                        let result =
                            serde_json::from_str::<ProvisionStateOutput>(&line);
                        formatter
                            .write_result(writer, &args, &result, &line)?;

                        result.map_err(|e| e.into())
                    })
                    .collect::<Vec<Result<_, _>>>()),
                Err(e) => {
                    // provisioning as a whole failed for this plugin
                    // TODO: decide format...
                    writeln!(
                        writer,
                        "plugin failed provisioning {}: {}",
                        p.definition.name, e
                    )?;
                    Err(e)
                }
            }
        })
        .flatten_ok()
        .collect::<Result<Vec<Result<_, Box<dyn std::error::Error>>>, _>>();

    // TODO: list unmatched states

    // TODO: ugh...
    if provision_results.is_err()
        || provision_results.unwrap().iter().any(Result::is_err)
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

        match Validator::new(&json_schema) {
            Ok(schema) => {
                for state in states {
                    let json_state = serde_json::to_value(state.state.clone())?;

                    if !schema.is_valid(&json_state) {
                        for error in schema.iter_errors(&json_state) {
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
