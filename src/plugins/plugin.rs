use crate::{
    config::{ConfigPlugin, PluginSource},
    eval::DeclaredState,
    extensions::SerdeDeserializeFromYamlPath,
    state::Condition,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};
use serde_yaml::Mapping;
use std::{
    ffi::OsStr,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PluginDefinition {
    pub name: String,
    pub executable: String,
    pub provision: PluginProvisionDefinition, // TODO: likely this will be optional (once we support var plugins...)

    #[serde_as(deserialize_as = "OneOrMany<_>")]
    #[serde(default)]
    pub when: Vec<Condition>,

    // TODO: could also support before: if necessary
    #[serde_as(deserialize_as = "OneOrMany<_>")]
    #[serde(default)]
    pub after: Vec<String>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
// TODO: maybe bad name...
pub struct PluginProvisionDefinition {
    #[serde_as(deserialize_as = "OneOrMany<_>")]
    #[serde(default)]
    pub when: Vec<Condition>,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Plugin {
    // TODO: maybe rename, represents the plugin on disk, currently same thing
    // as plugin source, but could be different if we add remote plugins that
    // need to be downloaded first
    pub path: PathBuf,
    pub definition: PluginDefinition,
}

// TODO: might change to be a specific plugin implementation (for basic plugins)
impl Plugin {
    pub fn from_config(plugin: &ConfigPlugin) -> Result<Plugin, Box<dyn std::error::Error>> {
        let path = match &plugin.source {
            PluginSource::Local { source } => source,
        };

        let definition = {
            let plugin_yml = path.join("plugin.yml");
            match PluginDefinition::from_yaml_file(&plugin_yml) {
                Ok(val) => Ok(val),
                Err(e) => Err(format!("{}: {}", e, plugin_yml.display())),
            }?
        };

        Ok(Plugin {
            path: path.to_path_buf(),
            definition,
        })
    }

    pub fn provision<'a>(
        &self,
        info: &ProvisionInfo,
        states: &Vec<DeclaredState>,
    ) -> Result<
        impl Iterator<Item = Result<ProvisionStateOutput, serde_json::Error>> + 'a,
        Box<dyn std::error::Error>,
    > {
        let info_json = serde_json::to_string(info)?;

        let mut child = self.execute(["provision", info_json.as_str()])?;

        // write states & close
        {
            let states_json = serde_json::to_string(states)?;

            let mut child_stdin = child
                .stdin
                .take()
                .ok_or("couldn't connect to plugin stdin")?;
            child_stdin.write_all(states_json.as_bytes())?;
        }

        let stdout = child
            .stdout
            .take()
            .ok_or("couldn't connect to plugin stdout")?;

        // TODO: include plugin information in iterator?
        // TODO: do something with stderr (include in iterator & log in error states?)
        Ok(serde_json::Deserializer::from_reader(stdout).into_iter::<ProvisionStateOutput>())
    }

    fn execute<I, S>(&self, args: I) -> std::io::Result<std::process::Child>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let plugin_executable = self.get_executable_path();

        Command::new(plugin_executable)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }

    fn get_executable_path(&self) -> PathBuf {
        self.path.join(&self.definition.executable)
    }
}

// TODO: move
#[derive(Deserialize, Debug, Clone)]
pub struct ProvisionStateOutput {
    pub status: ProvisionStateStatus,
    pub changed: bool,
    pub description: String,
    pub output: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ProvisionStateStatus {
    Failed,
    Success,
}

#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct ProvisionInfo {
    pub sources: Vec<PathBuf>,
    pub vars: Mapping,
}
