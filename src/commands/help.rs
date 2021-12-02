// TODO: consider splitting "args:" by command
const USAGE: &str = indoc::indoc! {"
    usage: nk [<global>...] <command> [<args>...]
      nk p|provision [--dry-run]
      nk h|help
      nk v|version

    global:
      -h|--help            Display this help message.
      -v|--version         Display the version.
      -c|--config <config> Override the config file.

    args:
      --dry-run Go through provisioning steps without actually applying anything.
"};

pub fn help() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", USAGE);
    Ok(())
}
