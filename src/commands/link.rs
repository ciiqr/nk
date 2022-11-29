use crate::{extensions::SerdeDeserializeFromYamlPath, plugins::PluginDefinition};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug)]
pub struct LinkArgs {
    pub path: PathBuf,
}

pub fn link(args: LinkArgs) -> Result<(), Box<dyn std::error::Error>> {
    // resolve path
    let canonical_path = match args.path.canonicalize() {
        Ok(p) => Ok(p),
        Err(e) => Err(format!("{}: {}", e, args.path.display())),
    }?;

    // load plugin info
    let definition = {
        let plugin_yml = args.path.join("plugin.yml");
        match PluginDefinition::from_yaml_file(&plugin_yml) {
            Ok(val) => Ok(val),
            Err(e) => Err(format!("{}: {}", e, plugin_yml.display())),
        }?
    };

    let plugin_dir = PathBuf::from_str(&shellexpand::tilde(
        format!("~/.nk/plugins/{}", definition.name).as_str(),
    ))?;

    // delete existing plugin dir
    if plugin_dir.try_exists()? {
        fs::remove_dir_all(plugin_dir.clone())?;
    }

    // make parent dir
    let parent_dir = plugin_dir
        .parent()
        .ok_or("plugin destination parent dir could not be determined")?;
    fs::create_dir_all(parent_dir)?;

    // link plugin
    symlink_dir(canonical_path, plugin_dir)?;

    Ok(())
}

#[cfg(unix)]
pub fn symlink_dir<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> std::io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

#[cfg(windows)]
pub fn symlink_dir<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(original, link)
}
