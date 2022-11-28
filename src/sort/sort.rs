use std::collections::HashMap;

use itertools::Itertools;
use topological_sort::TopologicalSort;

use crate::eval::ExecutionSets;

pub fn sort_execution_sets(execution_sets: &mut ExecutionSets) {
    // collect plugin names by the declarations they provision
    let mut plugin_names_by_declaration: HashMap<String, Vec<String>> = HashMap::new();
    for (plugin, states) in &*execution_sets {
        let declarations = states
            .iter()
            .map(|s| s.declaration.clone())
            .unique()
            .collect::<Vec<_>>();

        for declaration in declarations {
            let new_plugin_names = match plugin_names_by_declaration.remove(&declaration) {
                Some(mut plugin_names) => {
                    plugin_names.push(plugin.definition.name.clone());
                    plugin_names
                }
                None => vec![plugin.definition.name.clone()],
            };

            plugin_names_by_declaration.insert(declaration, new_plugin_names);
        }
    }

    // sort plugin names topologically
    let mut ts = TopologicalSort::<String>::new();
    for (plugin, _) in &*execution_sets {
        ts.insert(plugin.definition.name.clone());

        for declaration in &plugin.definition.after {
            // TODO: debugging level log if we don't match
            if let Some(dependent_plugin_names) = plugin_names_by_declaration.get(declaration) {
                for dependent_plugin_name in dependent_plugin_names {
                    ts.add_dependency(dependent_plugin_name, plugin.definition.name.clone());
                }
            }
        }
    }

    // sort independent plugins based on their order in the config
    let mut plugin_order = ts.pop_all();
    plugin_order.sort_by_cached_key(|name| {
        execution_sets
            .iter()
            .map(|(p, _)| p)
            .find(|p| p.definition.name == *name)
            .unwrap_or_else(|| unreachable!("plugin not found in execution sets"))
            .config_index
    });

    // append dependent plugins
    // TODO: order of dependent plugins is still variable, may need to implement our own topological sort
    let mut dependent_plugins = ts.collect::<Vec<_>>();
    plugin_order.append(&mut dependent_plugins);

    // sort based on plugin names
    execution_sets.sort_by_cached_key(|(plugin, _)| {
        plugin_order
            .iter()
            .position(|p| *p == plugin.definition.name)
            .unwrap_or_else(|| unreachable!("plugin position not found in plugin order list"))
    });
}
