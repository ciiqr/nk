#[cfg(not(unix))]
pub fn ensure_not_root() -> Result<(), String> {
    Ok(())
}
#[cfg(unix)]
pub fn ensure_not_root() -> Result<(), String> {
    unsafe {
        if nix::libc::getuid() == 0 {
            Err("nk: should not be run as root".into())
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
    use std::process::Command;

    let status = Command::new("sudo").args(["true"]).status()?;

    if status.success() {
        Ok(())
    } else {
        Err("nk: failed to run sudo".into())
    }
}
