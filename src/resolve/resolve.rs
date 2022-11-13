use std::collections::HashMap;

use crate::{
    config::Config,
    eval::Evaluator,
    merge::merge_groups,
    render::render_group,
    state::{self, ResolvedGroup},
};
use serde_yaml::Value;

pub struct ResolveOptions {
    pub render: bool,
}

pub fn resolve(
    config: &Config,
    builtin_vars: &HashMap<String, Value>,
    evaluator: &Evaluator,
    options: ResolveOptions,
) -> Result<ResolvedGroup, Box<dyn std::error::Error>> {
    // find all state files for this machine
    let files = state::File::find_all(&config.sources)?;

    // filter groups based on conditions
    let groups = evaluator.filter_files_to_matching_groups(&files)?;

    // merge all groups into into single resolved state
    let resolved = groups.into_iter().fold(ResolvedGroup::new(), merge_groups);

    // render resolved
    Ok(match options.render {
        true => render_group(builtin_vars.clone(), resolved)?,
        false => resolved,
    })
}
