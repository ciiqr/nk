use crate::state;

// TODO: wrap most errors in our own, more user friendly error
pub fn provision(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: need to solidify:
    // - how we determine config directories (config priority? cli args, ./nk.yml, ~/.nk.yml, ? /etc/nk.yml)
    //   - sources: [~/Projects/config, ~/Projects/config-private]
    // - how roles are organized {source}/{role}/
    // - how roles are merged (only if relevant, only by same name...)
    // - how we define the roles to include (machine.yml? cli args? ~/.* config? some combination of these)
    //   - if config:
    //     roles: [base, frontend, development, frontend]
    //     machine: server-data # assumes there are machine.yml files in the sources? or linked some other way?
    //   - if config, maybe we have a subcommand to generate this file based on passed in args

    // TODO: with the above decided, change from just loading this one file
    let file = state::File::from_path(expand_user("~/Projects/nk/sample.yml"))?;
    println!("{:#?}", file);

    // TODO: consider improving broken pipe error:
    // write!(std::io::stdout(), "{:#?}", file)?;
    // https://github.com/rust-lang/rust/issues/46016#issuecomment-605624865
    // https://crates.io/crates/nix

    // TODO: remove debug
    // let mut output = String::new();
    // for config in yaml_documents {
    //     {
    //         // NOTE: YamlEmitter doesn't support writing multiple docs properly
    //         // - thus we scope it so we can write a newline between each doc
    //         let mut emitter = YamlEmitter::new(&mut output);
    //         emitter.multiline_strings(true);

    //         // TODO: dump has result, don't ignore it...
    //         emitter.dump(&config)?;
    //     }
    //     output.write_str("\n")?;
    // }
    // println!("{}", output);

    println!("TODO: implement provision: dry_run={}", dry_run);

    Ok(())
}

// TODO: move
fn expand_user(path: &str) -> String {
    // TODO: maybe want a more specific outcome, atm this will complain about no such file or directory
    if let Ok(home) = std::env::var("HOME") {
        return path.replace("~", &home);
    }

    return path.into();
}
