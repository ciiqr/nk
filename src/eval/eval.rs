use crate::{
    plugins::Plugin,
    state::{self, Condition},
};
use rhai::{Engine, Scope};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_yaml::Value;
use std::collections::HashMap;

pub struct Evaluator {
    engine: Engine,
}

impl Evaluator {
    pub fn new(vars: HashMap<&str, Value>) -> Self {
        let mut global_scope = Scope::new();
        for (k, var) in vars {
            match var {
                Value::Null => global_scope.push_constant(k, ()),
                Value::Bool(v) => global_scope.push_constant(k, v),
                Value::Number(v) => global_scope.push_constant(k, v),
                Value::String(v) => global_scope.push_constant(k, v),
                Value::Sequence(v) => global_scope.push_constant(k, v),
                Value::Mapping(v) => global_scope.push_constant(k, v),
                Value::Tagged(v) => global_scope.push_constant(k, v),
            };
        }

        // setup engine
        let mut engine = Engine::new();
        #[allow(deprecated)]
        engine.on_var(move |name, _index, _context| {
            Ok(global_scope.get_value(name))
        });

        Evaluator { engine }
    }

    pub fn eval_conditions(
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
                match self.eval_conditions(&g.when, &mut rhai::Scope::new()) {
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

        Ok(execution_sets.into_iter().map(|(p, v)| (p, v)).collect())
    }
}

// TODO: really should be fixing the Value type...
pub type ExecutionSets = Vec<(Plugin, Vec<DeclaredState>)>;

// TODO: rename?
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct DeclaredState {
    pub declaration: String,
    pub state: Value,
}
