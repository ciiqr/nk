use crate::{
    config::{ConfigPlugin, PluginSource},
    extensions::SerdeDeserializeFromYamlPath,
    state::Condition,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};
use std::{
    ffi::OsStr,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
// TODO: maybe bad name...
pub struct PluginDefinition {
    pub name: String,
    pub executable: String,

    #[serde_as(deserialize_as = "OneOrMany<_>")]
    #[serde(default)]
    pub when: Vec<Condition>,
    // TODO: will likely have a priority system (ie. so files are created first, then programs are installed, then everything else?)
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

    // pub fn bootstrap(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     // TODO: consider an option to make bootstrapping optional (ie. an "implements" key in the plugin yaml with the subcommands it's implemented...)
    //     let child = self.execute(["bootstrap"])?;

    //     // TODO: handle errors smoother
    //     let output = child.wait_with_output()?;
    //     // TODO: need a proper error
    //     assert!(output.status.success());

    //     // TODO: print stdout/stderr? as applicable
    //     // let mut me = output.stdout.as_mut().unwrap();
    //     println!("output: {:#?}", output);
    //     // println!("stdout: {:?}", String::from_utf8(output.stdout));

    //     Ok(())
    // }

    // TODO: pass in states
    // TODO: forward to plugin through stdin
    pub fn provision(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut child = self.execute(["provision"])?;

        // write state
        {
            let child_stdin = child
                .stdin
                .as_mut()
                .ok_or("couldn't connect to plugin stdin")?;
            child_stdin.write_all("Hello, world!\n".to_string().as_bytes())?;
        }

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
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }

    fn get_executable_path(&self) -> PathBuf {
        self.path.join(&self.definition.executable)
    }
}
