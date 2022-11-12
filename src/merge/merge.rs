use std::collections::HashMap;

use serde_yaml::{Mapping, Value};

use crate::state::{self, ResolvedGroup};

pub fn merge_groups(mut a: ResolvedGroup, b: state::Group) -> ResolvedGroup {
    for (k, v) in b.declarations {
        let declaration = match a.declarations.remove(&k) {
            Some(d) => merge_declarations(d, v),
            None => v,
        };

        a.declarations.insert(k, declaration);
    }

    for (k, v) in b.vars {
        let var = match a.vars.remove(&k) {
            Some(d) => merge_values(d, v),
            None => v,
        };

        a.vars.insert(k, var);
    }

    a
}

fn merge_declarations(mut a: state::Declaration, mut b: state::Declaration) -> state::Declaration {
    a.states.append(&mut b.states);
    a
}

fn merge_values(a: Value, b: Value) -> Value {
    match (a, b) {
        (Value::Mapping(mut a), Value::Mapping(b)) => {
            for (k, v) in b {
                let var = match a.remove(&k) {
                    Some(d) => merge_values(d, v),
                    None => v,
                };

                a.insert(k, var);
            }
            Value::Mapping(a)
        }
        // TODO: decide how we want to handle lists...
        (_, b) => b,
    }
}

pub fn merge_vars(builtin_vars: HashMap<String, Value>, vars: Mapping) -> Result<Mapping, String> {
    // NOTE: if we end up with any built in mapping vars, we may want to merge this properly
    let mut data = Mapping::new();
    data.extend(builtin_vars.into_iter().map(|(k, v)| (Value::String(k), v)));
    data.extend(vars.into_iter());

    Ok(data)
}
