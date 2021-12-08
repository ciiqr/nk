use std::path::PathBuf;

#[derive(Debug)]
pub struct Role {
    pub name: String,
    pub sources: Vec<PathBuf>,
}

impl Role {
    pub fn find_by_names(role_names: &[String], sources: &[PathBuf]) -> Vec<Role> {
        role_names
            .iter()
            .map(|role_name| Role {
                name: role_name.into(),
                sources: find_role_sources(role_name, sources),
            })
            .collect()
    }
}

fn find_role_sources(role_name: &str, sources: &[PathBuf]) -> Vec<PathBuf> {
    sources
        .iter()
        .map(|source| source.join(role_name))
        .filter(|role_path| role_path.is_dir())
        .collect()
}
