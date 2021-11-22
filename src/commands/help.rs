const USAGE: &str = indoc::indoc! {"
    usage: nk [-h|--help] [-v|--version] <command> [<args>...]
      nk p|provision [--dry-run]
      nk h|help
      nk v|version
"};

pub fn help() -> Result<(), Box<dyn std::error::Error>> {
    Ok(println!("{}", USAGE))
}
