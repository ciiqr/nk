use crate::{config::Config, state::Machine};
use home::home_dir;
use os_info::Type;
use serde_yaml::Value;
use std::process::Command;
use std::{collections::HashMap, env};
use strum::{Display, EnumIter, EnumString};

pub struct SystemVars {
    pub distro: SystemDistro,
    pub os: SystemOs,
    pub family: SystemFamily,
    pub arch: SystemArch,
}

pub fn get_system_vars() -> Result<SystemVars, Box<dyn std::error::Error>> {
    Ok(SystemVars {
        distro: get_system_distro()?,
        os: get_system_os()?,
        family: get_system_family()?,
        arch: get_system_arch()?,
    })
}

// TODO: consider converting to a struct
pub fn get_builtin_vars(
    config: &Config,
) -> Result<HashMap<&str, Value>, Box<dyn std::error::Error>> {
    let SystemVars {
        distro,
        os,
        family,
        arch,
    } = get_system_vars()?;
    let hostname = hostname::get()?.to_string_lossy().to_string();
    let Machine {
        name: machine,
        roles,
    } = get_current_machine(config, &hostname)?;

    Ok(HashMap::from([
        // system vars
        ("os", os.to_string().into()),
        ("family", family.to_string().into()),
        ("arch", arch.to_string().into()),
        ("distro", distro.to_string().into()),
        // config vars
        ("hostname", hostname.into()),
        ("machine", machine.into()),
        ("roles", get_roles(roles).into()),
        ("user", whoami::username().into()),
        ("home", get_home_dir()?.into()),
    ]))
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

#[derive(Clone, Copy, EnumIter, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum SystemDistro {
    Alpine,
    Amazon,
    Android,
    Arch,
    Centos,
    Debian,
    Dragonfly,
    Emscripten,
    Endeavouros,
    Fedora,
    Freebsd,
    Garuda,
    Gentoo,
    Hardenedbsd,
    Illumos,
    Linux,
    Manjaro,
    Mariner,
    Midnightbsd,
    Mint,
    Netbsd,
    Nixos,
    Openbsd,
    Opensuse,
    Oracle,
    Pop,
    Raspbian,
    Redhat,
    RedhatEnterprise,
    Redox,
    Solus,
    Suse,
    Ubuntu,
    Windows,
    // macos
    Sonoma,
    Ventura,
    Monterey,
    BigSur,
    Catalina,
    Mojave,
    HighSierra,
    Sierra,
    // unknown
    Unknown,
}

fn get_system_distro() -> Result<SystemDistro, Box<dyn std::error::Error>> {
    Ok(match os_info::get().os_type() {
        Type::Alpine => SystemDistro::Alpine,
        Type::Amazon => SystemDistro::Amazon,
        Type::Android => SystemDistro::Android,
        Type::Arch => SystemDistro::Arch,
        Type::CentOS => SystemDistro::Centos,
        Type::Debian => SystemDistro::Debian,
        Type::DragonFly => SystemDistro::Dragonfly,
        Type::Emscripten => SystemDistro::Emscripten,
        Type::EndeavourOS => SystemDistro::Endeavouros,
        Type::Fedora => SystemDistro::Fedora,
        Type::FreeBSD => SystemDistro::Freebsd,
        Type::Garuda => SystemDistro::Garuda,
        Type::Gentoo => SystemDistro::Gentoo,
        Type::HardenedBSD => SystemDistro::Hardenedbsd,
        Type::Illumos => SystemDistro::Illumos,
        Type::Linux => SystemDistro::Linux,
        Type::Macos => get_macos_distro()?,
        Type::Manjaro => SystemDistro::Manjaro,
        Type::Mariner => SystemDistro::Mariner,
        Type::MidnightBSD => SystemDistro::Midnightbsd,
        Type::Mint => SystemDistro::Mint,
        Type::NetBSD => SystemDistro::Netbsd,
        Type::NixOS => SystemDistro::Nixos,
        Type::OpenBSD => SystemDistro::Openbsd,
        Type::openSUSE => SystemDistro::Opensuse,
        Type::OracleLinux => SystemDistro::Oracle,
        Type::Pop => SystemDistro::Pop,
        Type::Raspbian => SystemDistro::Raspbian,
        Type::Redhat => SystemDistro::Redhat,
        Type::RedHatEnterprise => SystemDistro::RedhatEnterprise,
        Type::Redox => SystemDistro::Redox,
        Type::Solus => SystemDistro::Solus,
        Type::SUSE => SystemDistro::Suse,
        Type::Ubuntu => SystemDistro::Ubuntu,
        Type::Windows => SystemDistro::Windows,
        _ => SystemDistro::Unknown,
    })
}

fn get_macos_distro() -> Result<SystemDistro, Box<dyn std::error::Error>> {
    let result = Command::new("sw_vers").args(["-productVersion"]).output()?;
    let output = String::from_utf8(result.stdout)?;
    let version_string = output.trim_end();
    let version = version_string.split('.').collect::<Vec<_>>();

    match version[..] {
        ["14", ..] => Ok(SystemDistro::Sonoma),
        ["13", ..] => Ok(SystemDistro::Ventura),
        ["12", ..] => Ok(SystemDistro::Monterey),
        ["11", ..] => Ok(SystemDistro::BigSur),
        ["10", "15", ..] => Ok(SystemDistro::Catalina),
        ["10", "14", ..] => Ok(SystemDistro::Mojave),
        ["10", "13", ..] => Ok(SystemDistro::HighSierra),
        ["10", "12", ..] => Ok(SystemDistro::Sierra),
        _ => Err(format!("unrecognized version: {}", version_string).into()),
    }
}

#[derive(Clone, Copy, EnumIter, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum SystemOs {
    Linux,
    Macos,
    Windows,
}

fn get_system_os() -> Result<SystemOs, Box<dyn std::error::Error>> {
    match env::consts::OS {
        "linux" => Ok(SystemOs::Linux),
        "macos" => Ok(SystemOs::Macos),
        "windows" => Ok(SystemOs::Windows),
        os => Err(format!("unsupported os: {os}").into()),
    }
}

#[derive(Clone, Copy, EnumIter, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum SystemFamily {
    Unix,
    Windows,
}

fn get_system_family() -> Result<SystemFamily, Box<dyn std::error::Error>> {
    match env::consts::FAMILY {
        "unix" => Ok(SystemFamily::Unix),
        "windows" => Ok(SystemFamily::Windows),
        os => Err(format!("unsupported os family: {os}").into()),
    }
}

#[derive(Clone, Copy, EnumIter, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum SystemArch {
    X86_64,
    Aarch64,
}

fn get_system_arch() -> Result<SystemArch, Box<dyn std::error::Error>> {
    match env::consts::ARCH {
        "x86_64" => Ok(SystemArch::X86_64),
        "aarch64" => Ok(SystemArch::Aarch64),
        os => Err(format!("unsupported os arch: {os}").into()),
    }
}
