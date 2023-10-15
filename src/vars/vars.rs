use home::home_dir;
use os_info::Type;
use serde::Serialize;
use serde_yaml::{Mapping, Value};
use std::env;
use std::process::Command;
use strum::{Display, EnumIter, EnumString};

pub struct SystemVars {
    pub distro: String,
    pub os: String,
    pub family: String,
    pub arch: String,
}

pub fn get_system_vars() -> Result<SystemVars, Box<dyn std::error::Error>> {
    Ok(SystemVars {
        distro: get_system_distro()?.to_string(),
        os: get_system_os()?.to_string(),
        family: get_system_family()?.to_string(),
        arch: get_system_arch()?.to_string(),
    })
}

#[derive(Serialize, Debug)]
pub struct BuiltinVars {
    // system vars
    pub distro: String,
    pub os: String,
    pub family: String,
    pub arch: String,

    // user vars
    pub hostname: String,
    pub machine: String,
    pub roles: Vec<String>,
    pub user: String,
    pub home: String,
}

impl BuiltinVars {
    pub fn to_mapping(&self) -> Mapping {
        let Value::Mapping(res) = serde_yaml::to_value(self)
            .expect("BuiltinVars.serialize should not return errors")
        else {
            unreachable!("BuiltinVars should always serialize to a mapping");
        };

        res
    }
}

// TODO: consider including sources as a var (could then change ProvisionInfo to just be vars...)
pub fn get_builtin_vars() -> Result<BuiltinVars, Box<dyn std::error::Error>> {
    let SystemVars {
        distro,
        os,
        family,
        arch,
    } = get_system_vars()?;
    let hostname = hostname::get()?.to_string_lossy().to_string();

    Ok(BuiltinVars {
        distro,
        os,
        family,
        arch,
        hostname: hostname.clone(),
        machine: hostname,
        roles: vec![],
        user: whoami::username(),
        home: get_home_dir()?,
    })
}

fn get_home_dir() -> Result<String, String> {
    Ok(home_dir()
        .ok_or("Could not determine home directory")?
        .as_os_str()
        .to_string_lossy()
        .into())
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
