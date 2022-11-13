use home::home_dir;
use os_info::Type;
use serde_yaml::Value;
use std::collections::HashMap;

use crate::{config::Config, state};

pub fn get_builtin_vars(
    config: &Config,
) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
    // determine machine/role information
    let machine = state::Machine::get_current(config)?;

    let mut vars = HashMap::new();

    vars.insert("machine".into(), Value::String(machine.name.clone()));
    vars.insert(
        "roles".into(),
        Value::Sequence(machine.roles.into_iter().map(Value::String).collect()),
    );
    vars.insert("os".into(), Value::String(std::env::consts::OS.into()));
    vars.insert(
        "family".into(),
        Value::String(std::env::consts::FAMILY.into()),
    );
    vars.insert("arch".into(), Value::String(std::env::consts::ARCH.into()));
    vars.insert("user".into(), Value::String(whoami::username()));
    vars.insert(
        "home".into(),
        Value::String(
            home_dir()
                .ok_or("Could not determine home directory")?
                .as_os_str()
                .to_string_lossy()
                .into_owned(),
        ),
    );
    vars.insert("distro".into(), get_distro_var());

    Ok(vars)
}

fn get_distro_var() -> Value {
    let info = os_info::get();

    let distro = match info.os_type() {
        Type::Alpine => "alpine",
        Type::Amazon => "amazon",
        Type::Android => "android",
        Type::Arch => "arch",
        Type::CentOS => "centos",
        Type::Debian => "debian",
        Type::DragonFly => "dragonfly",
        Type::Emscripten => "emscripten",
        Type::EndeavourOS => "endeavouros",
        Type::Fedora => "fedora",
        Type::FreeBSD => "freebsd",
        Type::Garuda => "garuda",
        Type::Gentoo => "gentoo",
        Type::HardenedBSD => "hardenedbsd",
        Type::Illumos => "illumos",
        Type::Linux => "linux",
        Type::Macos => "macos",
        Type::Manjaro => "manjaro",
        Type::Mariner => "mariner",
        Type::MidnightBSD => "midnightbsd",
        Type::Mint => "mint",
        Type::NetBSD => "netbsd",
        Type::NixOS => "nixos",
        Type::OpenBSD => "openbsd",
        Type::openSUSE => "opensuse",
        Type::OracleLinux => "oracle",
        Type::Pop => "pop",
        Type::Raspbian => "raspbian",
        Type::Redhat => "redhat",
        Type::RedHatEnterprise => "redhat_enterprise",
        Type::Redox => "redox",
        Type::Solus => "solus",
        Type::SUSE => "suse",
        Type::Ubuntu => "ubuntu",
        Type::Windows => "windows",
        Type::Unknown => "unknown",
        _ => "unknown",
    };

    Value::String(distro.into())
}
