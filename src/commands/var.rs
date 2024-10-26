use serde_yml::Value;

use crate::{args::VarSetArgs, vars::Globals};

pub fn var_set(args: VarSetArgs) -> Result<(), Box<dyn std::error::Error>> {
    // parse value
    let value: Value = serde_yml::from_str(&args.value)?;

    // load existing globals
    let mut globals = Globals::load()?;

    // set new value
    globals.vars.insert(args.name.into(), value);

    // save globals
    globals.save()?;

    Ok(())
}
