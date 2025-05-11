#![warn(missing_docs)]
//! A library for modding Electron apps in-memory, without modifying any program files.
//!
//! This library was made for improving the modding experience for Discord, but it can be used for any Electron app.
//!
//! # Features
//!
//! - `asar`: Enables the ASAR archive builder. (enabled by default)
//! - `uuid`: Enables the use of random UUIDs for ASAR archive names. (enabled by default)
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
//!     require(process.env.MODLOADER_MOD_ENTRYPOINT).inject(asar);
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
pub mod paths;

// For Linux
#[cfg(target_os = "linux")]
mod linux;

// For Windows
// TODO: Re-implement Windows support.
#[cfg(target_os = "windows")]
mod windows;

// TODO: For MacOS

/// Launches an Electron executable with the provided information.
///
/// `id` on Linux: the path to the executable.
///
/// `id` on Windows: the path to the directory containing `Update.exe`.
///
/// `library_path`: The path to the electron-hook `.so` or `.dll`
///
/// `asar_path`: The path to the ASAR file to inject
///
/// `args`: Arguments to pass to the executable
///
/// `detach`: It is recommended to set `detach` to true to prevent the process from dying when the parent process is closed.
#[allow(unused_variables)]
pub fn launch(
    executable: &str,
    library_path: &str,
    asar_path: &str,
    args: Vec<String>,
    detach: bool,
) -> Result<Option<u32>, String> {
    #[cfg(target_os = "linux")]
    {
        linux::launch(executable, library_path, asar_path, args, detach)
    }

    #[cfg(target_os = "windows")]
    {
        // No need for detach on Windows, as the process already detaches itself.
        windows::launch(executable, library_path, asar_path, args)
    }
}

/// Launches an Electron executable through Flatpak with the provided information.
///
/// This is only available on Linux.
///
/// TODO: This only supports global packages. Are --user flatpak packages handled differently?
///
/// `id`: The ID of the flatpak package.
///
/// `library_path`: The path to the electron-hook `.so` or `.dll`
///
/// `asar_path`: The path to the ASAR file to inject
///
/// `args`: Arguments to pass to the executable
///
/// `detach`: It is recommended to set `detach` to true to prevent the process from dying when the parent process is closed.
#[cfg(any(doc, target_os = "linux"))]
pub fn launch_flatpak(
    id: &FlatpakID,
    library_path: &str,
    asar_path: &str,
    args: Vec<String>,
    detach: bool,
) -> Result<Option<u32>, String> {
    linux::launch_flatpak(id, library_path, asar_path, args, detach)
}

/// The ID of a Flatpak package.
pub enum FlatpakID {
    /// A User install of a flatpak package. Will be run with `--user`
    User(String),
    /// A System install of a flatpak package. Will be run with `--system`
    System(String),
}

impl std::fmt::Display for FlatpakID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlatpakID::User(id) => write!(f, "{id}"),
            FlatpakID::System(id) => write!(f, "{id}"),
        }
    }
}
