use crate::{
    plugins::{Plugin, PluginDefinitionPartial},
    state::{self, Condition},
};
use rhai::{serde::to_dynamic, Engine, Scope};
use serde::{Deserialize, Serialize};
use serde_yml::{Mapping, Value};
use std::collections::HashMap;

pub struct Evaluator {
    engine: Engine,
}

impl Evaluator {
    pub fn new(global_vars: Mapping) -> Self {
        // setup engine
        let mut engine = Engine::new();
        #[allow(deprecated)]
        engine.on_var(move |name, _index, _context| {
            Ok(match global_vars.get(name).unwrap_or(&Value::Null) {
                Value::Null => None,
                Value::Bool(v) => Some(to_dynamic(v)?),
                Value::Number(v) => Some(to_dynamic(v)?),
                Value::String(v) => Some(to_dynamic(v)?),
                Value::Sequence(v) => Some(to_dynamic(v)?),
                Value::Mapping(v) => Some(to_dynamic(v)?),
                Value::Tagged(v) => Some(to_dynamic(v)?),
            })
        });

        Evaluator { engine }
    }

    fn eval_conditions(
        &self,
        conditions: &[Condition],
        scope: &mut Scope,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        for condition in conditions {
            let condition_matches = self
                .engine
                .eval_expression_with_scope::<bool>(scope, &condition.rule)?;

            // if any conditions don't match, return early
            if !condition_matches {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn filter_files_to_matching_groups(
        &self,
        files: &[state::File],
    ) -> Result<Vec<state::Group>, Box<dyn std::error::Error>> {
        files
            .iter()
            .flat_map(|f| {
                f.groups
                    .clone()
                    .into_iter()
                    .map(|g| (g, f.path.clone()))
                    .collect::<Vec<(_, _)>>()
            })
            .filter_map(|(g, p)| {
                match self.eval_conditions(&g.when, &mut Scope::new()) {
                    Ok(false) => None,
                    Ok(true) => Some(Ok(g)),
                    Err(e) => Some(Err(format!(
                        "{}: in conditions {:?} of {}",
                        e,
                        g.when,
                        p.display()
                    )
                    .into())),
                }
            })
            .collect()
    }

    pub fn filter_plugin_partials(
        &self,
        partials: Vec<PluginDefinitionPartial>,
    ) -> Result<Vec<PluginDefinitionPartial>, Box<dyn std::error::Error>> {
        let mut filtered_partials = Vec::new();

        for partial in partials {
            let when = partial.when.clone().unwrap_or_default();

            match self.eval_conditions(&when, &mut Scope::new()) {
                Ok(true) => {
                    filtered_partials.push(partial);
                }
                Err(e) => Err(e)?,
                _ => (),
            }
        }

        Ok(filtered_partials)
    }

    pub fn filter_plugins(
        &self,
        plugins: Vec<Plugin>,
    ) -> Result<Vec<Plugin>, Box<dyn std::error::Error>> {
        let mut filtered_plugins = Vec::new();

        for plugin in plugins {
            match self
                .eval_conditions(&plugin.definition.when, &mut Scope::new())
            {
                Ok(true) => {
                    filtered_plugins.push(plugin);
                }
                Err(e) => Err(e)?,
                _ => (),
            }
        }

        Ok(filtered_plugins)
    }

    pub fn match_states_to_plugins(
        &self,
        declarations: &HashMap<String, state::Declaration>,
        plugins: &[Plugin],
    ) -> Result<ExecutionSets, Box<dyn std::error::Error>> {
        let mut execution_sets: HashMap<Plugin, Vec<DeclaredState>> =
            HashMap::new();

        for declaration in declarations.values() {
            for state in declaration.states.clone() {
                let mut scope = Scope::new();
                scope.push_constant("declaration", declaration.name.clone());
                scope.push_constant_dynamic(
                    "state",
                    rhai::serde::to_dynamic(state.clone())?,
                );

                // TODO: clean up this code
                let mut matching_plugin = None;
                for plugin in plugins {
                    match self.eval_conditions(
                        &plugin.definition.provision.when,
                        &mut scope,
                    ) {
                        Ok(true) => {
                            matching_plugin = Some(plugin);
                            break;
                        }
                        Err(e) => Err(e)?,
                        _ => (),
                    }
                }
                if let Some(plugin) = matching_plugin {
                    let declared_state = DeclaredState {
                        declaration: declaration.name.clone(),
                        state,
                    };
                    if let Some((_, v)) =
                        execution_sets.iter_mut().find(|(p, _)| *p == plugin)
                    {
                        v.push(declared_state);
                    } else {
                        execution_sets
                            .insert(plugin.clone(), vec![declared_state]);
                    }
                } else {
                    // TODO: would prefer to handle the logging for this in provision
                    println!("unmatched: {}: {:?}", declaration.name, state);
                }
            }
        }

        Ok(execution_sets.into_iter().collect())
    }

    pub fn filter_execution_sets(
        &self,
        execution_sets: &mut ExecutionSets,
        filter: &str,
    ) {
        // filter execution_sets
        for (_, states) in execution_sets.iter_mut() {
            states.retain(|state| {
                // TODO: add plugin info?
                let mut scope = Scope::new();
                scope.push_constant("declaration", state.declaration.clone());
                scope.push_constant_dynamic(
                    "state",
                    // TODO: not sure if this expect is k...
                    rhai::serde::to_dynamic(state.state.clone())
                        .expect("state to be rhai serializable"),
                );

                let res = self
                    .engine
                    .eval_expression_with_scope::<bool>(&mut scope, filter);

                // TODO: if all states produce an error, show one of them (to help debug strictly invalid filters)
                // NOTE: errors are treated as the filter not matching
                matches!(res, Ok(true))
            });
        }

        // remove any empty execution sets
        execution_sets.retain(|(_, states)| !states.is_empty());
    }
}

// TODO: really should be fixing the Value type...
pub type ExecutionSets = Vec<(Plugin, Vec<DeclaredState>)>;

// TODO: rename?
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct DeclaredState {
    pub declaration: String,
    pub state: Value,
}
