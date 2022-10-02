use rhai::{Engine, Scope};

use crate::state::{self, Condition, Machine};

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
        mut scope: Scope,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let results = conditions
            .iter()
            .map(|c| self.engine.eval_with_scope::<bool>(&mut scope, &c.rule))
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
                |(g, p)| match self.eval_conditions(&g.when, rhai::Scope::new()) {
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
}
