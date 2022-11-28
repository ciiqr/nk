use crate::{
    config::Config, eval::Evaluator, plugins::load_plugins, resolve::resolve as resolveState,
    resolve::ResolveOptions, vars::get_builtin_vars,
};

#[derive(Debug)]
pub enum ResolveOutputFormat {
    Yaml,
    Json,
}
#[derive(Debug)]
pub struct ResolveArgs {
    pub output: ResolveOutputFormat,
    pub render: bool,
}

// TODO: wrap most errors in our own, more user friendly error
pub async fn resolve(args: ResolveArgs, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // initialize builtin vars
    let builtin_vars = get_builtin_vars(&config)?;

    // initialize evaluator (machine, roles, platform, etc.)
    let evaluator = Evaluator::new(builtin_vars.clone());

    // load plugins
    let plugins = load_plugins(&config, &builtin_vars, &evaluator).await?;

    // resolve state
    let resolved = resolveState(
        &config,
        &builtin_vars,
        &evaluator,
        &plugins,
        ResolveOptions {
            render: args.render,
        },
    )?;

    // print state
    match args.output {
        ResolveOutputFormat::Yaml => print!("{}", serde_yaml::to_string(&resolved)?),
        ResolveOutputFormat::Json => println!("{}", serde_json::to_string(&resolved)?),
    };

    Ok(())
}
