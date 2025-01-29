mod hooks;

pub use hooks::*;

#[cfg(target_os = "linux")]
pub(crate) fn launch(
    executable: &str,
    shared_object_path: &str,
    asar_path: &str,
    args: Vec<String>,
    profile_directory: Option<String>,
    detach: bool,
) -> Result<std::process::ExitStatus, String> {
    let executable = std::path::PathBuf::from(executable);
    let profile_directory = profile_directory.map(std::path::PathBuf::from);

    // Detach the process from the parent. This prevents the application from dying when the parent process (e.g. terminal) is closed.
    if detach {
        unsafe { libc::setsid() };
    }

    let working_dir = if let Some(ref profile_directory) = profile_directory {
        profile_directory
    } else {
        executable
            .parent()
            .ok_or("Failed to get parent directory from executable")?
    };

    let Ok(mut target) = std::process::Command::new(&executable)
        .current_dir(working_dir)
        .env("LD_PRELOAD", shared_object_path)
        .env("MODLOADER_ASAR_PATH", asar_path)
        .args(args)
        .spawn()
    else {
        return Err("Failed to launch instance".into());
    };

    let Ok(exit_status) = target.wait() else {
        return Err("Process exited unexpectedly".into());
    };

    Ok(exit_status)
}
