use crate::{
    args::{ResolveArgs, ResolveOutputFormat},
    config::Config,
    eval::Evaluator,
    plugins::load_plugins,
    resolve::resolve as resolveState,
    resolve::ResolveOptions,
    vars::get_global_vars,
};

// TODO: wrap most errors in our own, more user friendly error
pub async fn resolve(
    args: ResolveArgs,
    config: Config,
) -> Result<(), Box<dyn std::error::Error>> {
    // initialize global vars
    let global_vars = get_global_vars()?;

    // initialize evaluator
    let evaluator = Evaluator::new(global_vars.clone());

    // load plugins
    let plugins = load_plugins(&config, &evaluator).await?;

    // resolve state
    let resolved = resolveState(
        &config,
        &global_vars,
        &evaluator,
        &plugins,
        &ResolveOptions {
            render: args.render,
        },
    )?;

    // print state
    match args.output {
        ResolveOutputFormat::Yaml => {
            print!("{}", serde_yaml::to_string(&resolved)?);
        }
        ResolveOutputFormat::Json => {
            println!("{}", serde_json::to_string(&resolved)?);
        }
    };

    Ok(())
}
