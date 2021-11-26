use crate::state;

pub struct ProvisionArgs {
    pub dry_run: bool,
}

// TODO: wrap most errors in our own, more user friendly error
pub fn provision(args: ProvisionArgs) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: load from sources instead of static file
    let file = state::File::from_path(expand_user("~/Projects/nk/sample.yml"))?;
    println!("{:#?}", file);

    // TODO: consider improving broken pipe error:
    // write!(std::io::stdout(), "{:#?}", file)?;
    // https://github.com/rust-lang/rust/issues/46016#issuecomment-605624865
    // https://crates.io/crates/nix

    println!("TODO: implement provision: dry_run={}", args.dry_run);

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
