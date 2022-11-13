use handlebars::{Handlebars, RenderError};
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;

use crate::{
    merge::merge_vars,
    state::{Declaration, ResolvedGroup},
};

struct TemplatingEngine<'reg> {
    registry: Handlebars<'reg>,
    data: Mapping,
}

impl<'reg> TemplatingEngine<'reg> {
    fn new(data: Mapping) -> Self {
        let mut registry = Handlebars::new();
        registry.set_strict_mode(true);

        Self { registry, data }
    }

    fn render(&self, template: &str) -> Result<String, RenderError> {
        self.registry.render_template(template, &self.data)
    }
}

pub fn render_group(
    builtin_vars: HashMap<String, Value>,
    group: ResolvedGroup,
) -> Result<ResolvedGroup, Box<dyn std::error::Error>> {
    let data = merge_vars(builtin_vars, group.vars.clone())?;
    let engine = TemplatingEngine::new(data);

    let declarations = group
        .declarations
        .into_iter()
        .map(|(k, d)| Ok((k, render_declaration(&engine, d)?)))
        .collect::<Result<_, Box<dyn std::error::Error>>>()?;

    Ok(ResolvedGroup {
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
