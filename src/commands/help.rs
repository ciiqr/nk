// TODO: consider splitting "args:" by command
const USAGE: &str = indoc::indoc! {"
    usage: nk [<global>...] <command> [<args>...]
      nk p|provision [--dry-run] [--show-unchanged]
      nk h|help
      nk v|version
      nk plugin bash

    global:
      -h|--help            Display this help message.
      -v|--version         Display the version.
      -c|--config <config> Override the config file.

    args:
      --dry-run        Run through provisioning without applying any changes.
      --show-unchanged Whether to print unchanges results.
"};

pub fn help() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", USAGE);
    Ok(())
}
