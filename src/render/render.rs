use handlebars::{Handlebars, RenderError};
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;

use crate::state::{Declaration, ResolvedGroup};

pub struct RenderedGroup {
    pub vars: Mapping,
    pub declarations: HashMap<String, Declaration>,
}

struct TemplatingEngine<'reg> {
    registry: Handlebars<'reg>,
    data: Mapping,
}

impl<'reg> TemplatingEngine<'reg> {
    fn new(data: Mapping) -> Self {
        Self {
            registry: Handlebars::new(),
            data,
        }
    }

    fn render(&self, template: &str) -> Result<String, RenderError> {
        self.registry.render_template(template, &self.data)
    }
}

fn build_data(vars: Mapping) -> Mapping {
    // TODO: share values with eval.rs
    let mut data = Mapping::new();
    data.insert(
        Value::String("user".into()),
        Value::String(whoami::username()),
    );
    data.extend(vars.into_iter());

    data
}

pub fn render_group(group: ResolvedGroup) -> Result<RenderedGroup, Box<dyn std::error::Error>> {
    let data = build_data(group.vars.clone());
    let engine = TemplatingEngine::new(data);

    let declarations = group
        .declarations
        .into_iter()
        .map(|(k, d)| Ok((k, render_declaration(&engine, d)?)))
        .collect::<Result<_, Box<dyn std::error::Error>>>()?;

    Ok(RenderedGroup {
        vars: group.vars,
        declarations,
    })
}

fn render_declaration(
    engine: &TemplatingEngine,
    declaration: Declaration,
) -> Result<Declaration, Box<dyn std::error::Error>> {
    let states = declaration
        .states
        .into_iter()
        .map(|s| render_state(engine, s))
        .collect::<Result<_, _>>()?;

    Ok(Declaration {
        name: declaration.name,
        states,
    })
}

fn render_state(
    engine: &TemplatingEngine,
    state: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    match state {
        Value::String(v) => Ok(Value::String(engine.render(&v)?)),
        Value::Sequence(v) => Ok(v
            .into_iter()
            .map(|s| render_state(engine, s))
            .collect::<Result<Value, _>>()?),
        Value::Mapping(v) => Ok(Value::Mapping(
            v.into_iter()
                .map(|(k, s)| Ok((k, render_state(engine, s)?)))
                .collect::<Result<_, Box<dyn std::error::Error>>>()?,
        )),
        v => Ok(v),
    }
}
