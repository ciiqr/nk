use std::collections::HashMap;

use rhai::{Engine, Scope};

use crate::{
    plugins::Plugin,
    state::{self, Condition, Machine},
};

pub struct Evaluator {
    engine: Engine,
    // global_scope: Scope<'a>,
}

impl Evaluator {
    pub fn new(machine: &Machine) -> Self {
        // TODO: consider how I want to handle this stuff (maybe make lazy, maybe include in Evaluator with Rc?)
        let mut global_scope = Scope::new();
        global_scope.push_constant("machine", machine.name.clone());
        global_scope.push_constant("roles", machine.roles.clone());
        global_scope.push_constant("os", std::env::consts::OS);
        global_scope.push_constant("family", std::env::consts::FAMILY);
        global_scope.push_constant("arch", std::env::consts::ARCH);

        // setup engine
        let mut engine = Engine::new();
        #[allow(deprecated)]
        engine.on_var(move |name, _index, _context| Ok(global_scope.get_value(name)));

        Evaluator {
            engine,
            // global_scope,
        }
    }

    pub fn eval_conditions(
        &self,
        conditions: &[Condition],
        scope: &mut Scope,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let results = conditions
            .iter()
            .map(|c| self.engine.eval_with_scope::<bool>(scope, &c.rule))
            .collect::<Result<Vec<bool>, _>>()?;

        Ok(results.iter().all(|r| *r))
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
            .filter_map(
                |(g, p)| match self.eval_conditions(&g.when, &mut rhai::Scope::new()) {
                    Ok(false) => None,
                    Ok(true) => Some(Ok(g)),
                    Err(e) => Some(Err(format!(
                        "{}: in conditions {:?} of {}",
                        e,
                        g.when,
                        p.display()
                    )
                    .into())),
                },
            )
            .collect()
    }

    pub fn match_states_to_plugins(
        &self,
        declarations: &HashMap<String, state::Declaration>,
        plugins: Vec<Plugin>,
    ) -> Result<ExecutionSets, Box<dyn std::error::Error>> {
        let mut execution_sets: HashMap<Plugin, Vec<serde_yaml::Value>> = HashMap::new();

        for declaration in declarations.values() {
            for state in declaration.states.clone() {
                let mut scope = Scope::new();
                scope.push_constant("declaration", declaration.name.clone());
                // TODO: this doesn't quite work because we can't access fields on state...
                // scope.push_constant("state", state.clone());

                // TODO: clean up this code
                let mut matching_plugin = None;
                for plugin in &plugins {
                    match self.eval_conditions(&plugin.definition.when, &mut scope) {
                        Ok(true) => {
                            matching_plugin = Some(plugin);
                            break;
                        }
                        Err(e) => Err(e)?,
                        _ => (),
                    }
                }
                if let Some(plugin) = matching_plugin {
                    if let Some((_, v)) = execution_sets.iter_mut().find(|(p, _)| *p == plugin) {
                        v.push(state);
                    } else {
                        execution_sets.insert(plugin.clone(), vec![state]);
                    }
                } else {
                    // TODO: decide what to do if no plugins match, at least log
                    // println!("unmatched: {}: {:?}", declaration.name, state);
                }
            }
        }

        Ok(execution_sets.into_iter().map(|(p, v)| (p, v)).collect())
    }
}

// TODO: really should be fixing the Value type...
type ExecutionSets = Vec<(Plugin, Vec<serde_yaml::Value>)>;
