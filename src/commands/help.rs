// TODO: consider splitting "args:" by command
const USAGE: &str = indoc::indoc! {"
    usage: nk [<global>...] <command> [<args>...]
      nk p|provision [--show-unchanged]
      nk r|resolve [--no-render] [--output yaml|json]
      nk h|help
      nk v|version
      nk plugin bash

    global:
      -h|--help            Display this help message.
      -v|--version         Display the version.
      -c|--config <config> Override the config file.

    args:
      --show-unchanged Whether to print unchanges results.
      --no-render      To disable rendering of templated values.
      --output         Output format.
"};

pub fn help() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", USAGE);
    Ok(())
}
