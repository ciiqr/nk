use super::{Condition, Declaration};
use yaml_rust::Yaml;

#[derive(Debug)]
pub struct Group {
    pub when: Vec<Condition>,
    // TODO: tags? vars? (if they're not handled via state...)
    pub declarations: Vec<Declaration>,
}

impl Group {
    const WHEN: &'static str = "when";
    const KEYWORDS: [&'static str; 1] = [Group::WHEN];

    pub fn from_yaml(yaml: &Yaml) -> Result<Group, String> {
        Ok(Group {
            when: parse_group_when(&yaml[Group::WHEN])?,
            declarations: parse_group_declarations(&yaml)?,
        })
    }
}

fn parse_group_when(yaml: &Yaml) -> Result<Vec<Condition>, String> {
    match yaml {
        Yaml::Array(yamls) => yamls.iter().map(|y| Condition::from_yaml(&y)).collect(),
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
