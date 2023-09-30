use crate::{config::Config, state::Machine};
use home::home_dir;
use os_info::Type;
use serde_yaml::Value;
use std::process::Command;
use std::{collections::HashMap, env};

pub fn get_builtin_vars(
    config: &Config,
) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
    // determine machine/role information
    let hostname = hostname::get()?.to_string_lossy().to_string();
    let machine = get_current_machine(config, &hostname)?;

    let mut vars = HashMap::new();

    vars.insert("hostname".into(), Value::String(hostname));
    vars.insert("machine".into(), Value::String(machine.name));
    vars.insert("roles".into(), Value::Sequence(get_roles(machine.roles)));
    vars.insert("os".into(), Value::String(env::consts::OS.into()));
    vars.insert("family".into(), Value::String(env::consts::FAMILY.into()));
    vars.insert("arch".into(), Value::String(env::consts::ARCH.into()));
    vars.insert("user".into(), Value::String(whoami::username()));
    vars.insert("home".into(), Value::String(get_home_dir()?));
    vars.insert("distro".into(), get_distro_var()?);

    Ok(vars)
}

fn get_roles(roles: Vec<String>) -> Vec<Value> {
    roles.into_iter().map(Value::String).collect()
}

fn get_home_dir() -> Result<String, String> {
    Ok(home_dir()
        .ok_or("Could not determine home directory")?
        .as_os_str()
        .to_string_lossy()
        .into())
}

fn get_current_machine(
    config: &Config,
    hostname: &str,
) -> Result<Machine, Box<dyn std::error::Error>> {
    let machine = config.machine.clone().unwrap_or_else(|| hostname.into());

    Ok(config
        .machines
        .iter()
        .find(|m| m.name == machine)
        .cloned()
        .unwrap_or(Machine {
            name: machine,
            roles: vec![],
        }))
}

fn get_distro_var() -> Result<Value, Box<dyn std::error::Error>> {
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
        Type::Macos => get_macos_distro()?,
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
        _ => "unknown",
    };

    Ok(Value::String(distro.into()))
}

fn get_macos_distro() -> Result<&'static str, Box<dyn std::error::Error>> {
    let result = Command::new("sw_vers").args(["-productVersion"]).output()?;
    let output = String::from_utf8(result.stdout)?;
    let version_string = output.trim_end();
    let version = version_string.split('.').collect::<Vec<_>>();

    match version[..] {
        ["14", ..] => Ok("sonoma"),
        ["13", ..] => Ok("ventura"),
        ["12", ..] => Ok("monterey"),
        ["11", ..] => Ok("big_sur"),
        ["10", "15", ..] => Ok("catalina"),
        ["10", "14", ..] => Ok("mojave"),
        ["10", "13", ..] => Ok("high_sierra"),
        ["10", "12", ..] => Ok("sierra"),
        _ => Err(format!("unrecognized version: {}", version_string).into()),
    }
}
