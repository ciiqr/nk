#[cfg(not(unix))]
pub fn ensure_not_root() -> Result<(), String> {
    Ok(())
}
#[cfg(unix)]
pub fn ensure_not_root() -> Result<(), String> {
    unsafe {
        if nix::libc::getuid() == 0 {
            Err("should not be run as root".into())
        } else {
            Ok(())
        }
    }
}

#[cfg(not(unix))]
pub fn sudo_prompt() -> Result<(), String> {
    Ok(())
}

#[cfg(unix)]
pub fn sudo_prompt() -> Result<(), Box<dyn std::error::Error>> {
    use console::style;
    use std::process::Command;

    let status = Command::new("sudo").args(["true"]).status()?;
    if !status.success() {
        return Err("failed to run sudo".into());
    }

    // now attempt non-interactive
    let status = Command::new("sudo").args(["-n", "true"]).status()?;
    if !status.success() {
        return Err(format!("{}\n\techo \"Defaults:${{USER}} timestamp_timeout=30\" | sudo EDITOR='tee -a' visudo \"/etc/sudoers.d/0-${{USER}}-timeout\"",
            style("requires either a reasonable sudo timeout (see timestamp_timeout) or passwordless sudo. ie. try setting a 30m timeout:").red().bold()
        ).into());
    }

    Ok(())
}
