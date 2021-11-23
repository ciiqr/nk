use std::fmt::Write;

use yaml_rust::{YamlEmitter, YamlLoader};

// TODO: wrap most errors in our own, more user friendly error
pub fn provision(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: need to solidify:
    // - how we determine config directories (some sort of ~/.* config AND/OR cli args)
    // - how roles are organized
    // - how we define the roles to include (machine.yml? cli args? ~/.* config? some combination of these)
    // TODO: with the above decided, change from just loading this one file
    let contents = std::fs::read_to_string(expand_user("~/nk.yml"))?;
    let configs = YamlLoader::load_from_str(&contents)?;

    // config

    // TODO: remove debug
    let mut output = String::new();

    for config in configs {
        {
            // NOTE: YamlEmitter doesn't support writing multiple docs properly
            // - thus we scope it so we can write a newline between each doc
            let mut emitter = YamlEmitter::new(&mut output);
            emitter.multiline_strings(true);

            // TODO: dump has result, don't ignore it...
            emitter.dump(&config)?;
        }
        output.write_str("\n")?;
    }

    println!("{}", output);

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
