use crate::state;

pub fn merge_groups(a: state::Group, b: &state::Group) -> state::Group {
    let mut declarations = a.declarations;
    for (k, mut v) in b.declarations.clone() {
        let declaration = match declarations.remove(&k) {
            Some(d) => merge_declarations(d, &mut v),
            None => v,
        };

        declarations.insert(k, declaration);
    }

    state::Group {
        // TODO: a little dumb since we're using groups here still... maybe introduce new type that excludes "when"
        when: vec![],
        declarations,
    }
}

fn merge_declarations(mut a: state::Declaration, b: &mut state::Declaration) -> state::Declaration {
    a.states.append(&mut b.states);

    state::Declaration {
        name: b.name.clone(),
        states: a.states,
    }
}
