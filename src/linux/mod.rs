mod hooks;

use super::FlatpakID;

pub(crate) fn launch_flatpak(
    id: &FlatpakID,
    library_path: &str,
    asar_path: &str,
    args: Vec<String>,
    detach: bool,
) -> Result<Option<u32>, String> {
    // If the library starts with /home, return the whole path.
    // If the library is absolute, prefix wtih /run/host
    // If it's a local file, provide the whole path
    // Otherwise, assume it's in /usr/lib, so prefix with /run/host/usr/lib
    let library_path = if library_path.starts_with("/home") {
        library_path.to_string()
    } else if library_path.starts_with('/') {
        format!("/run/host{}", library_path)
    } else {
        let current_dir = std::env::current_dir().unwrap();
        let local_path = current_dir.join(&library_path);

        if local_path.is_file() {
            local_path.to_string_lossy().into_owned()
        } else {
            format!("/run/host/usr/lib/{}", library_path)
        }
    };

    let asar_dir = std::path::PathBuf::from(asar_path);
    let asar_dir = asar_dir
        .parent()
        .ok_or("Failed to get parent directory from asar path")?
        .to_string_lossy();

    let mod_entrypoint = std::env::var("MODLOADER_MOD_ENTRYPOINT")
        .map_err(|_| "MODLOADER_MOD_ENTRYPOINT not set")?;

    let mod_entrypoint_dir = std::path::PathBuf::from(mod_entrypoint);
    let mod_entrypoint_dir = mod_entrypoint_dir
        .parent()
        .ok_or("Failed to get parent directory from mod entrypoint")?
        .to_string_lossy();

    let current_executable = std::env::current_exe().unwrap().display().to_string();

    let current_executable = if current_executable.starts_with("/usr") {
        format!("/run/host{}", current_executable)
    } else {
        current_executable
    };

    let mut target = std::process::Command::new("flatpak");

    target.arg("run");

    match id {
        FlatpakID::User(_) => target.arg("--user"),
        FlatpakID::System(_) => target.arg("--system"),
    };

    target
        .arg("--filesystem=host:ro") // allows us to read /usr/lib as /run/host/usr/lib
        .arg(format!("--filesystem={}:ro", asar_dir)) // Read-only access to the ASAR dir
        .arg(format!("--filesystem={}:create", mod_entrypoint_dir)) // let the mod update itself...
        .arg(format!("--filesystem={}:ro", current_executable))
        .arg(format!("--env=ZYPAK_LD_PRELOAD={}", library_path)) // give zypak our LD_PRELOAD
        .arg(format!("--env=MODLOADER_ASAR_PATH={}", asar_path))
        .arg(format!("--env=MODLOADER_EXECUTABLE={}", current_executable))
        .arg(format!("--env=MODLOADER_LIBRARY_PATH={}", library_path))
        .arg(format!(
            "--env=MODLOADER_ORIGINAL_ASAR_RELATIVE=../_app.asar"
        ))
        .arg(id.to_string())
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

    let process_args = std::env::args().skip(1).collect::<Vec<String>>();
    let process_args_json = serde_json::to_string(&process_args).unwrap_or_else(|_| "[]".into());

    target
        .current_dir(working_dir)
        .env("LD_PRELOAD", library_path)
        .env("MODLOADER_ASAR_PATH", asar_path)
        .env("MODLOADER_EXECUTABLE", std::env::current_exe().unwrap())
        .env("MODLOADER_LIBRARY_PATH", library_path)
        .env("MODLOADER_ORIGINAL_ASAR_RELATIVE", "../_app.asar")
        .env("MODLOADER_PROCESS_ARGV", process_args_json)
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
