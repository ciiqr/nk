use crate::{
    args::LinkArgs, extensions::SerdeDeserializeFromYamlPath,
    plugins::PluginDefinition,
};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

pub fn link(args: &LinkArgs) -> Result<(), Box<dyn std::error::Error>> {
    // create nk plugin directory
    let nk_plugins_dir =
        PathBuf::from_str(&shellexpand::tilde("~/.nk/plugins"))?;
    fs::create_dir_all(&nk_plugins_dir)?;

    for path in &args.paths {
        // load plugin info
        let definition = match PluginDefinition::from_yaml_file(path) {
            Ok(val) => Ok(val),
            Err(e) => Err(format!("{}: {}", e, path.display())),
        }?;

        // delete existing plugin dir
        let plugin_dir = nk_plugins_dir.join(definition.name);
        if plugin_dir.try_exists()? {
            fs::remove_dir_all(&plugin_dir)?;
        }

        // resolve path to plugin.yml
        let canonical_path = match path.canonicalize() {
            Ok(p) => Ok(p),
            Err(e) => Err(format!("{}: {}", e, path.display())),
        }?;
        // get parent of plugin.yml
        let canonical_parent = canonical_path
            .parent()
            .ok_or("could not determine plugin parent")?;

        // link plugin
        symlink_dir(canonical_parent, plugin_dir)?;
    }

    Ok(())
}

#[cfg(unix)]
pub fn symlink_dir<P: AsRef<Path>, Q: AsRef<Path>>(
    original: P,
    link: Q,
) -> std::io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

#[cfg(windows)]
pub fn symlink_dir<P: AsRef<Path>, Q: AsRef<Path>>(
    original: P,
    link: Q,
) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(original, link)
}
