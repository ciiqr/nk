use crate::{
    args::PackArgs,
    plugins::{Manifest, ManifestAssets, ManifestPlugin, PluginFile},
    state::Condition,
    vars::{SystemArch, SystemDistro, SystemFamily, SystemOs},
};
use flate2::{write::GzEncoder, Compression};
use itertools::Itertools;
use std::fs::{self, OpenOptions};
use strum::IntoEnumIterator;
use tar::Builder;

pub fn pack(args: PackArgs) -> Result<(), Box<dyn std::error::Error>> {
    // create output directory
    fs::create_dir_all(&args.output)?;

    let mut manifest = Manifest {
        owner: args.owner,
        repo: args.repo,
        version: args.version,
        plugins: Vec::with_capacity(args.paths.len()),
    };

    for plugin_yml_path in args.paths {
        // load plugin info
        let plugin_file = PluginFile::from_yaml_file(&plugin_yml_path)?;

        // get name
        let names: Vec<_> = plugin_file
            .partials
            .iter()
            .filter_map(|p| p.name.clone())
            .unique()
            .collect();
        if names.len() > 1 {
            return Err("multiple names not currently supported".into());
        }
        let name = names.first().ok_or("missing required field: name")?;

        // resolve path to plugin.yml
        let canonical_path = match plugin_yml_path.canonicalize() {
            Ok(p) => Ok(p),
            Err(e) => Err(format!("{}: {}", e, plugin_yml_path.display())),
        }?;
        // get parent of plugin.yml
        let canonical_parent = canonical_path
            .parent()
            .ok_or("could not determine plugin parent")?;

        let mut assets = vec![];
        for partial in plugin_file.partials {
            let when = partial.when.unwrap_or_default();

            // NOTE: each partial with an executable is packed on its own
            if let Some(executable) = partial.executable {
                // determine asset filename
                let filename = get_asset_filename(name, &when);
                let file = format!("{filename}.tar.gz");

                // generate tar.gz
                let tar_file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(args.output.join(&file))
                    .map_err(|e| {
                        format!(
                            "{e}: out path \"{}\" should be writable...",
                            args.output.display()
                        )
                    })?;
                let encoder = GzEncoder::new(tar_file, Compression::default());
                let mut tar = Builder::new(encoder);

                // append executable & plugin.yml to tar
                // TODO: probably need a way of packaging additional files
                tar.append_path_with_name(
                    canonical_parent.join(&executable),
                    executable,
                )?;
                tar.append_path_with_name(&plugin_yml_path, "plugin.yml")?;

                // append asset
                assets.push(ManifestAssets { file, when });
            }
        }

        // append plugin to manifest
        manifest.plugins.push(ManifestPlugin {
            name: name.clone(),
            assets,
        });
    }

    // write manifest.yml
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(args.output.join("manifest.yml"))
        .map_err(|e| {
            format!(
                "{e}: out path \"{}\" should be writable...",
                args.output.display()
            )
        })?;
    serde_yaml::to_writer(file, &manifest).map_err(|e| {
        format!(
            "{e}: out path \"{}\" should be writable...",
            args.output.display()
        )
    })?;

    Ok(())
}

// TODO: explore better ways of solving this...
fn get_asset_filename(name: &str, when: &[Condition]) -> String {
    // NOTE: this only handles simple `var == "value"` conditions for system vars:
    // ie.
    // - `os == "macos"` -> {name}-macos
    // - [`os == "macos"`, `arch == "aarch64"`] -> {name}-macos-aarch64
    // - `family == "unix"` -> {name}-unix

    let conditions = when.iter().map(|c| &c.rule).collect::<Vec<_>>();
    let mut filename_parts = vec![name.to_string()];

    // distro/os/family are mutually exclusive since each is more specific than the next
    if let Some((distro, _)) = SystemDistro::iter()
        .map(|distro| (distro, format!("distro == \"{}\"", distro)))
        .find(|(_, condition)| conditions.contains(&condition))
    {
        // distro
        filename_parts.push(distro.to_string());
    } else if let Some((os, _)) = SystemOs::iter()
        .map(|os| (os, format!("os == \"{}\"", os)))
        .find(|(_, condition)| conditions.contains(&condition))
    {
        // os
        filename_parts.push(os.to_string());
    } else if let Some((family, _)) = SystemFamily::iter()
        .map(|family| (family, format!("family == \"{}\"", family)))
        .find(|(_, condition)| conditions.contains(&condition))
    {
        // family
        filename_parts.push(family.to_string());
    }

    // arch
    if let Some((arch, _)) = SystemArch::iter()
        .map(|arch| (arch, format!("arch == \"{}\"", arch)))
        .find(|(_, condition)| conditions.contains(&condition))
    {
        filename_parts.push(arch.to_string());
    }

    filename_parts.join("-")
}
