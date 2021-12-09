use crate::{
    config::{ConfigPlugin, PluginSource},
    extensions::SerdeDeserializeFromYamlPath,
    state::Condition,
};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Command, Stdio},
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
// TODO: maybe bad name...
pub struct PluginDefinition {
    name: String,
    executable: String,
    #[serde(default)]
    when: Vec<Condition>,
}

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
            PluginDefinition::from_yaml_file(&plugin_yml)?
        };

        Ok(Plugin {
            path: path.to_path_buf(),
            definition,
        })
    }

    // TODO: maybe this is part of Plugin creation process? or is implicitly run once something runs that needs it?
    // - maybe it can be run in from_config?
    pub fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let child = self.execute(["setup"])?;

        // TODO: handle errors smoother
        let output = child.wait_with_output()?;
        assert!(output.status.success());

        // TODO: print stdout/stderr? as applicable
        // let mut me = output.stdout.as_mut().unwrap();
        println!("output: {:#?}", output);
        // println!("stdout: {:?}", String::from_utf8(output.stdout));

        Ok(())
    }

    // TODO: pass in states
    pub fn provision(&self) -> Result<(), Box<dyn std::error::Error>> {
        let child = self.execute(["provision", "--state", "example state"])?;

        // TODO: handle errors smoother
        let output = child.wait_with_output()?;
        assert!(output.status.success());

        // TODO: print stdout/stderr? as applicable
        // let mut me = output.stdout.as_mut().unwrap();
        println!("output: {:#?}", output);
        // println!("stdout: {:?}", String::from_utf8(output.stdout));

        Ok(())
    }

    fn execute<I, S>(&self, args: I) -> std::io::Result<std::process::Child>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let plugin_executable = self.get_executable_path();

        Command::new(plugin_executable)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }

    fn get_executable_path(&self) -> PathBuf {
        self.path.join(&self.definition.executable)
    }
}
