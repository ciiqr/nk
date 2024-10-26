use crate::{
    eval::{DeclaredState, Evaluator},
    state::{Condition, Declaration, RawDeclaration},
    utils::{
        deserialize_map_to_map_of_named,
        deserialize_map_to_map_of_named_optional,
    },
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};
use serde_yml::{Mapping, Value};
use std::{
    collections::HashMap,
    ffi::OsStr,
    hash::{Hash, Hasher},
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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

    #[serde(
        default,
        deserialize_with = "deserialize_map_to_map_of_named::<RawDeclaration, _, _>"
    )]
    pub dependencies: HashMap<String, Declaration>,

    pub schema: Value,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct PluginDefinitionPartial {
    pub name: Option<String>,
    pub executable: Option<String>,
    pub provision: Option<PluginProvisionDefinition>,

    #[serde_as(deserialize_as = "Option<OneOrMany<_>>")]
    pub when: Option<Vec<Condition>>,

    #[serde_as(deserialize_as = "Option<OneOrMany<_>>")]
    pub after: Option<Vec<String>>,

    #[serde(
        default,
        deserialize_with = "deserialize_map_to_map_of_named_optional::<RawDeclaration, _, _>"
    )]
    pub dependencies: Option<HashMap<String, Declaration>>,

    pub schema: Option<Value>,
}

impl TryInto<PluginDefinition> for PluginDefinitionPartial {
    type Error = Box<dyn std::error::Error>;

    fn try_into(self) -> Result<PluginDefinition, Self::Error> {
        Ok(PluginDefinition {
            name: self.name.ok_or("missing required field, at least one matching partial must have: name")?,
            executable: self
                .executable
                .ok_or("missing required field, at least one matching partial must have: executable")?,
            provision: self
                .provision
                .ok_or("missing required field, at least one matching partial must have: provision")?,
            when: self.when.unwrap_or_default(),
            after: self.after.unwrap_or_default(),
            dependencies: self.dependencies.unwrap_or_default(),
            schema: self.schema.ok_or("missing required field, at least one matching partial must have: schema")?,
        })
    }
}

#[derive(Debug)]
pub struct PluginFile {
    pub partials: Vec<PluginDefinitionPartial>,
}

impl PluginFile {
    pub fn from_yaml_file(
        path: &PathBuf,
    ) -> Result<PluginFile, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;

        let partials = serde_yml::Deserializer::from_str(&contents)
            .map(PluginDefinitionPartial::deserialize)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PluginFile { partials })
    }
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

#[derive(Debug, Clone, Eq)]
pub struct Plugin {
    // TODO: maybe rename, represents the plugin on disk, currently same thing
    // as plugin source, but could be different if we add remote plugins that
    // need to be downloaded first
    pub path: PathBuf,
    pub definition: PluginDefinition,
    pub config_index: usize,
}

// TODO: path is good enough for now, but might want to compare more fields
// NOTE: custom implementation because HashMap is not itself hashable
impl PartialEq for Plugin {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Hash for Plugin {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.path.hash(hasher);
    }
}

// TODO: might change to be a specific plugin implementation (for basic plugins)
impl Plugin {
    pub fn from_path(
        path: PathBuf,
        config_index: usize,
        evaluator: &Evaluator,
    ) -> Result<Option<Plugin>, Box<dyn std::error::Error>> {
        let definition = {
            let plugin_yml = path.join("plugin.yml");

            // load plugin info
            let plugin_file = PluginFile::from_yaml_file(&plugin_yml)?;

            // filter partials
            let partials =
                evaluator.filter_plugin_partials(plugin_file.partials)?;

            // no partials match
            if partials.is_empty() {
                return Ok(None);
            }

            // merge partials
            let partial = partials
                .into_iter()
                .reduce(|mut acc, partial| {
                    if partial.name.is_some() {
                        acc.name = partial.name;
                    }
                    if partial.executable.is_some() {
                        acc.executable = partial.executable;
                    }
                    if partial.provision.is_some() {
                        acc.provision = partial.provision;
                    }
                    if partial.when.is_some() {
                        // TODO: merge instead... (doesn't matter atm, but might later)
                        acc.when = partial.when;
                    }
                    if partial.after.is_some() {
                        acc.after = partial.after;
                    }
                    if partial.dependencies.is_some() {
                        acc.dependencies = partial.dependencies;
                    }
                    if partial.schema.is_some() {
                        acc.schema = partial.schema;
                    }
                    acc
                })
                .unwrap_or_default();

            // partial into PluginDefinition
            partial.try_into()?
        };

        Ok(Some(Plugin {
            path,
            definition,
            config_index,
        }))
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
        Ok(serde_json::Deserializer::from_reader(stdout)
            .into_iter::<ProvisionStateOutput>())
    }

    fn execute<I, S>(
        &self,
        args: I,
    ) -> Result<std::process::Child, Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr> + std::fmt::Display,
    {
        let plugin_executable = self.get_executable_path();
        let plugin_executable_extension =
            plugin_executable.extension().unwrap_or_default();

        let mut command = if plugin_executable_extension == "ps1" {
            let mut cmd = Command::new("powershell");
            cmd.args([
                "-NonInteractive",
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
            ])
            .arg(format!(
                "$input | {} @args",
                plugin_executable.to_string_lossy()
            ))
            .args(
                // quote args with single quotes & replace inner single quotes with double the single quotes
                args.into_iter().map(|s| {
                    format!("'{}'", format!("{s}").replace('\'', "''"))
                }),
            );

            cmd
        } else {
            let mut cmd = Command::new(plugin_executable);
            cmd.args(args);
            cmd
        };

        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                format!("{}: {}", e, self.get_executable_path().display())
                    .into()
            })
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
#[serde(rename_all = "snake_case")]
pub enum ProvisionStateStatus {
    Failed,
    Success,
}

#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct ProvisionInfo {
    pub sources: Vec<PathBuf>,
    pub vars: Mapping,
}
