use yaml_rust::{Yaml, YamlLoader};

// TODO: move
#[derive(Debug)]
struct Condition {
    // TODO: decide if this is how we'll store this... maybe could parse and store parsed
    rule: String,
}
#[derive(Debug)]
struct Declaration {
    name: String,
    // TODO: instances OR states...
    instances: Vec<Yaml>,
}
#[derive(Debug)]
struct Group {
    when: Vec<Condition>,
    // TODO: tags? vars? (if they're not handled via state...)
    declarations: Vec<Declaration>,
}
struct StateFile {
    path: String,
    groups: Vec<Group>,
}

impl Condition {
    fn from_yaml(yaml: &Yaml) -> Result<Condition, String> {
        match yaml {
            Yaml::String(rule) => Ok(Condition { rule: rule.into() }),
            _ => Err("Invalid format for when: rule".into()),
        }
    }
}

impl Declaration {
    fn from_yaml(name: String, yaml: &Yaml) -> Result<Declaration, String> {
        Ok(Declaration {
            name,
            instances: match yaml {
                Yaml::Array(arr) => Ok(arr.to_vec()),

                // singletons are converted to an array of 1
                Yaml::Null
                | Yaml::Integer(_)
                | Yaml::String(_)
                | Yaml::Boolean(_)
                | Yaml::Hash(_)
                | Yaml::Real(_) => Ok(vec![yaml.to_owned()]),

                _ => Err("Invalid format for declaration"),
            }?,
        })
    }
}

impl Group {
    const WHEN: &'static str = "when";
    const KEYWORDS: [&'static str; 1] = [Group::WHEN];

    fn from_yaml(yaml: &Yaml) -> Result<Group, String> {
        Ok(Group {
            when: Group::parse_group_when(&yaml[Group::WHEN])?,
            declarations: Group::parse_group_declarations(&yaml)?,
        })
    }

    fn parse_group_when(yaml: &Yaml) -> Result<Vec<Condition>, String> {
        match yaml {
            Yaml::Array(yamls) => {
                let res: Result<Vec<Condition>, String> =
                    yamls.iter().map(|y| Condition::from_yaml(&y)).collect();
                res
            }
            Yaml::BadValue => Ok(vec![]),
            // TODO: make the errors context specific. Likely need to pass in details about current file? idk if we can get current line?
            _ => Err("Invalid format for when:".into()),
        }
    }

    fn parse_group_declarations(yaml: &Yaml) -> Result<Vec<Declaration>, String> {
        match yaml {
            Yaml::Hash(hash) => {
                let mut err: Result<(), String> = Ok(());

                let res = hash
                    .iter()
                    .filter_map(|(k, v)| match k {
                        Yaml::String(key) => Some((key, v)),
                        _ => {
                            // TODO: there's gotta be a cleaner way...
                            err = Err("Invalid type for top level key".into());
                            None
                        }
                    })
                    .filter(|(k, _)| !Group::KEYWORDS.contains(&k.as_str()))
                    .map(|(k, v)| Declaration::from_yaml(k.into(), v))
                    .collect();

                // Check for key type errors
                err?;
                res
            }
            Yaml::Null => Ok(vec![]), // allow empty files
            _ => Err(format!("Invalid format for top level declarations: {:#?}", yaml).into()),
        }
    }
}

impl StateFile {
    fn from_path(path: String) -> Result<StateFile, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(&path)?;
        let yaml_documents = YamlLoader::load_from_str(&contents)?;

        Ok(StateFile {
            path,
            groups: StateFile::parse_groups(&yaml_documents)?,
        })
    }

    fn parse_groups(yamls: &Vec<Yaml>) -> Result<Vec<Group>, String> {
        yamls.iter().map(|c| Group::from_yaml(&c)).collect()
    }
}

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
    let file = StateFile::from_path(expand_user("~/Projects/nk/sample.yml"))?;

    // print all
    println!("# {}", file.path);
    for group in file.groups {
        println!("{:#?}", group)
    }

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
