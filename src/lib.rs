#![warn(missing_docs)]
//! A library for modding Electron apps in-memory, without modifying any program files.
//!
//! This library was made for improving the modding experience for Discord, but it can be used for any Electron app.
//!
//! This library provides functionality to make the executable self-loadable as a shared-object (Linux) or DLL (Windows).
//! This makes it much more portable, without needing a separate .so or .dll file.
//! To use this, enable the `self-executable` feature.
//!
//! # Features
//!
//! - `asar`: Enables the ASAR archive builder. (enabled by default)
//! - `uuid`: Enables the use of random UUIDs for ASAR archive names. (enabled by default)
//! - `self-executable`: Makes the executable double as a shared library.
//!
//! # Examples
//!
//! electron-hook maps the original `app.asar` to `_app.asar`,
//! so keep this in mind if you need to call the original file anywhere,
//! as shown in this example.
//!
//! ```rust,ignore
//! use electron_hook::asar::Asar;
//!
//! let mod_dir = mod_artifact_dir("moonlight");
//!
//! let _download_url = "https://github.com/moonlight-mod/moonlight/releases/latest/download/dist.tar.gz";
//! // extract and save `_download_url` into `mod_dir`
//!
//! let mod_entrypoint = mod_dir.join("injector.js");
//!
//! let template = r#"
//!     console.log("Mod injected!!!");
//!     let asar = require("path").resolve(__dirname, "../_app.asar");
//!     require("$ENTRYPOINT").inject(asar);
//! "#;
//!
//! // Create the asar file
//! let asar = Asar::new()
//!     .with_id("moonlight")
//!     .with_template(template)
//!     .with_mod_entrypoint(mod_dir)
//!     .create()
//!     .unwrap();
//!
//! electron_hook::launch(
//!     "/path/to/executable/Discord",
//!     asar.path().to_str().unwrap(),
//!     vec!["--pass-arguments-here"],
//!     None, // Optional profile directory
//!     true, // Detach the process
//! );
//! ```

#[cfg(any(doc, feature = "asar"))]
pub mod asar;
pub mod macros;
pub mod paths;

// For Linux
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
#[allow(unused_imports)]
pub use linux::*;

// For Windows
// TODO: Re-implement Windows support.
// #[cfg(target_os = "windows")]
// #[allow(unused_imports)]
// mod windows;

// #[cfg(target_os = "windows")]
// #[allow(unused_imports)]
// pub use windows::*;

// TODO: For MacOS

/// Launches an Electron executable with the provided asar path, arguments, and optional profile directory.
///
/// It is recommended to set `detach` to true to prevent the process from dying when the parent process is closed.
///
/// This is only available when the `self-executable` feature is enabled.
/// If you want to use a separate file for the shared library, use `launch_with_shared` instead.
#[cfg(any(doc, all(unix, feature = "self-executable")))]
pub fn launch_with_self(
    executable: &str,
    asar_path: &str,
    args: Vec<String>,
    profile_directory: Option<String>,
    detach: bool,
) -> Result<std::process::ExitStatus, String> {
    let shared_object = std::env::current_exe().map_err(|_| "Failed to get current executable")?;
    launch_with_shared(
        executable,
        asar_path,
        shared_object.to_str().unwrap(),
        args,
        profile_directory,
        detach,
    )
}

/// Launches an Electron executable with the provided asar path, arguments, and optional profile directory.
///
/// It is recommended to set `detach` to true to prevent the process from dying when the parent process is closed.
///
/// This is only available when the `self-executable` feature is enabled.
/// If you want to use the executable as the shared library, use `launch_with_self` instead.
#[cfg(target_os = "linux")]
fn launch_with_shared(
    executable: &str,
    asar_path: &str,
    shared_object_path: &str,
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
            .parent()
            .ok_or("Failed to get parent directory from profile directory")?
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
