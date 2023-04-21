use std::collections::HashMap;

use crate::{
    config::Config,
    eval::Evaluator,
    merge::{merge_groups, merge_plugin_dependencies},
    plugins::Plugin,
    render::render_group,
    state::{self, ResolvedGroup},
};
use serde_yaml::{Mapping, Value};

pub struct ResolveOptions {
    pub render: bool,
}

pub fn resolve(
    config: &Config,
    builtin_vars: &HashMap<String, Value>,
    evaluator: &Evaluator,
    plugins: &[Plugin],
    options: ResolveOptions,
) -> Result<ResolvedGroup, Box<dyn std::error::Error>> {
    // find all state files for this machine
    let files = state::File::find_all(&config.sources)?;

    // filter groups based on conditions
    let groups = evaluator.filter_files_to_matching_groups(&files)?;

    // built in vars as mapping
    let mut vars = Mapping::new();
    vars.extend(
        builtin_vars
            .clone()
            .into_iter()
            .map(|(k, v)| (Value::String(k), v)),
    );

    // merge in plugin dependencies
    let resolved = plugins
        .iter()
        .flat_map(|p| p.definition.dependencies.clone().into_values())
        .fold(ResolvedGroup::new(vars), merge_plugin_dependencies);

    // merge all groups into into single resolved state
    let resolved = groups.into_iter().fold(resolved, merge_groups);

    // render resolved
    Ok(match options.render {
        true => render_group(resolved)?,
        false => resolved,
    })
}
