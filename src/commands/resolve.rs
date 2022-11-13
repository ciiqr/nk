use crate::{
    config::Config, eval::Evaluator, resolve::resolve as resolveState, resolve::ResolveOptions,
    vars::get_builtin_vars,
};

#[derive(Debug)]
pub struct ResolveArgs {
    // pub show_unchanged: bool,
    // TODO: --output yaml|json
    // TODO: --no-render | --render false
    pub render: bool,
}

// TODO: wrap most errors in our own, more user friendly error
pub fn resolve(args: ResolveArgs, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // initialize builtin vars
    let builtin_vars = get_builtin_vars(&config)?;

    // initialize evaluator (machine, roles, platform, etc.)
    let evaluator = Evaluator::new(builtin_vars.clone());

    // resolve state
    let resolved = resolveState(
        &config,
        &builtin_vars,
        &evaluator,
        ResolveOptions {
            render: args.render,
        },
    )?;

    // TODO: print state (properly)
    // serde_yaml::to_string(&resolved)?
    println!("{:#?}", resolved);

    Ok(())
}
