mod hooks;

#[cfg(target_os = "linux")]
pub(crate) fn launch(
    executable: &str,
    library_path: &str,
    asar_path: &str,
    args: Vec<String>,
    detach: bool,
) -> Result<Option<u32>, String> {
    let executable = std::path::PathBuf::from(executable);

    // Detach the process from the parent. This prevents the application from dying when the parent process (e.g. terminal) is closed.
    if detach {
        unsafe { libc::setsid() };
    }

    let working_dir = executable
        .parent()
        .ok_or("Failed to get parent directory from executable")?;

    let mut target = std::process::Command::new(&executable);

    target
        .current_dir(working_dir)
        .env("LD_PRELOAD", library_path)
        .env("MODLOADER_ASAR_PATH", asar_path)
        .env("MODLOADER_EXECUTABLE", std::env::current_exe().unwrap())
        .env("MODLOADER_LIBRARY_PATH", library_path)
        .env("MODLOADER_ORIGINAL_ASAR_RELATIVE", "../_app.asar")
        .args(args);

    // We also need to detach stdin.
    if detach {
        target
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .stdin(std::process::Stdio::null());
    };

    let Ok(mut target) = target.spawn() else {
        return Err("Failed to launch instance".into());
    };

    // If we aren't detaching, keep the process alive.
    if !detach {
        let Ok(_) = target.wait() else {
            return Err("Process exited unexpectedly".into());
        };

        return Ok(None);
    }

    let pid = target.id();

    Ok(Some(pid))
}
