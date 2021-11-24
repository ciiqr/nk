use yaml_rust::{Yaml, YamlLoader};

// TODO: move
// TODO: is there reason to remember StateFiles? (ie. to apply rules to the whole file)
// TODO: is there reason to remember StateRoles? (ie. to apply rules to the whole role)
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

// TODO: change error types?
fn parse_condition(yaml: &Yaml) -> Result<Condition, String> {
    match yaml {
        Yaml::String(rule) => Ok(Condition { rule: rule.into() }),
        _ => Err("Invalid format for when: rule".into()),
    }
}

fn parse_declaration(name: String, yaml: &Yaml) -> Result<Declaration, String> {
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

// TODO: can we get a filter_map with Result instead...? well really collect into a new iter
// - kinda want to just .collect() into a new iter
// .map(|(k, v)| match k {
//     Yaml::String(key) => Ok((key, v)),
//     _ => Err("TODO: Invalid format for declaration name"),
// })
// _ => Err("TODO: Invalid format for declaration name".into()),
const KEYWORDS: [&str; 1] = ["when"];
fn parse_group_declarations(yaml: &Yaml) -> Result<Vec<Declaration>, String> {
    match yaml {
        Yaml::Hash(hash) => hash
            .iter()
            .filter_map(|(k, v)| match k {
                Yaml::String(key) => Some((key, v)),
                // TODO: need an error or to figure out .collect().iter()
                _ => None,
            })
            .filter(|(k, _)| !KEYWORDS.contains(&k.as_str()))
            .map(|(k, v)| parse_declaration(k.into(), v))
            .collect(),
        Yaml::Null => Ok(vec![]), // allow empty files
        _ => Err(format!("Invalid format for top level declarations: {:#?}", yaml).into()),
    }
}

fn parse_group_when(yaml: &Yaml) -> Result<Vec<Condition>, String> {
    match yaml {
        Yaml::Array(yamls) => {
            let res: Result<Vec<Condition>, String> =
                yamls.iter().map(|y| parse_condition(&y)).collect();
            res
        }
        Yaml::BadValue => Ok(vec![]),
        // TODO: make the errors context specific. Likely need to pass in details about current file? idk if we can get current line?
        _ => Err("Invalid format for when:".into()),
    }
}

fn parse_group(yaml: &Yaml) -> Result<Group, String> {
    Ok(Group {
        when: parse_group_when(&yaml["when"])?,
        // TODO: would be nice to filter yaml here...
        declarations: parse_group_declarations(&yaml)?,
    })
}

fn parse_groups(yamls: &Vec<Yaml>) -> Result<Vec<Group>, String> {
    yamls.iter().map(|c| parse_group(&c)).collect()
}

// TODO: wrap most errors in our own, more user friendly error
pub fn provision(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: need to solidify:
    // - how we determine config directories (some sort of ~/.* config AND/OR cli args)
    // - how roles are organized
    // - how we define the roles to include (machine.yml? cli args? ~/.* config? some combination of these)
    // TODO: with the above decided, change from just loading this one file
    let contents = std::fs::read_to_string(expand_user("~/Projects/nk/sample.yml"))?;
    let yaml_documents = YamlLoader::load_from_str(&contents)?;

    let groups = parse_groups(&yaml_documents)?;

    // print all
    for group in groups {
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
