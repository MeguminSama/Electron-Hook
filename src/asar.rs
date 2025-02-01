//! Module for creating asar archives
//!
//! Provides an API for creating ASAR entrypoints from a template.
//!
//! This module requires the `asar` feature to be enabled.

fn make_package_json(wm_class: &Option<String>) -> String {
    if let Some(wm_class) = wm_class {
        format!(r#"{{"main": "index.js", "name": "{wm_class}"}}"#)
    } else {
        r#"{"main": "index.js"}"#.to_string()
    }
}

/// A builder for creating ASAR archives and writing them to the filesystem.
///
/// # Usage
///
/// ```rust,ignore
/// use electron_hook::asar::Asar;
/// use electron_hook::paths::{mod_artifact_dir, data_profile_dir};
///
/// let entrypoint = mod_artifact_dir("vencord").join("patcher.js");
/// let profile_dir = data_profile_dir("vencord");
///
/// let asar = Asar::new()
///     .with_id("vencord-release")
///     .with_template("require(process.env.MODLOADER_MOD_ENTRYPOINT);")
///     .with_mod_entrypoint(entrypoint.to_str().unwrap())
///     .with_profile_dir(profile_dir.to_str().unwrap()) // Optional
///     .create();
///
/// // Linux: /home/CoolPerson/.cache/electron-hook/asar/vencord-release.asar
/// // Windows: C:/Users/CoolPerson/AppData/Local/electron-hook/asar/vencord-release.asar
/// // MacOS: TODO
/// ```
#[derive(Debug, Default)]
pub struct Asar {
    /// The unique identifier for the ASAR archive.
    /// e.g. if this is set to `my-mod-name`, the final name will be `{id}.asar`.
    ///
    /// This can either be a random UUID (with the `uuid` feature) or a custom reusable ID.
    ///
    /// The final path will be something like:
    ///
    /// Linux: `/home/CoolPerson/.cache/electron-hook/asar/my-mod-name.asar`
    ///
    /// Windows: `C:/Users/CoolPerson/AppData/Local/electron-hook/asar/my-mod-name.asar`
    ///
    /// MacOS: TODO
    pub id: String,

    /// The template for the index.js that will go into the ASAR archive.
    ///
    /// There are multiple environment variables that can be used in the template:
    ///
    /// | Environment Variable               | Description                                     | Notes                    |
    /// | ---------------------------------- | ----------------------------------------------- | ------------------------ |
    /// | `MODLOADER_EXECUTABLE`             | Specifies the path to the Modloader executable. |                          |
    /// | `MODLOADER_MOD_ENTRYPOINT`         | The path to the entrypoint of the mod.          |                          |
    /// | `MODLOADER_ASAR_ID`                | The name of the ASAR ID selected                |                          |
    /// | `MODLOADER_ASAR_PATH`              | The path to the ASAR file.                      |                          |
    /// | `MODLOADER_LIBRARY_PATH`           | The path to the `.dll` or `.so`.                |                          |
    /// | `MODLOADER_ORIGINAL_ASAR_RELATIVE` | The relative path to the original ASAR file     | Is always `../_app.asar` |
    /// | `MODLOADER_PROFILE_DIR`            | The path to the custom profile directory        | Optional                 |
    /// | `MODLOADER_WM_CLASS`               | The WM_CLASS of the Electron application.       | Optional                 |
    /// | `MODLOADER_FOLDER_NAME`            | the app-<version> folder name                   | Windows only             |
    ///
    /// For a basic implementation, you want to at least require your mod, e.g.:
    ///
    /// ```javascript
    /// require(process.env.MODLOADER_MOD_ENTRYPOINT);
    /// ```
    pub template: String,

    /// The WM_CLASS of the application that the mod is for.
    ///
    /// You can use this to make it show as a different application on your Linux taskbar.
    ///
    /// This (probably) has no effect on Windows.
    pub wm_class: Option<String>,

    /// The entrypoint for the mod. This should be the path to the main file for your mod.
    ///
    /// Preferably, you should get the path using [electron_hook::paths::mod_artifact_dir]
    ///
    /// You can use it like so:
    ///
    /// ```rust
    /// use electron_hook::paths::mod_artifact_dir;
    /// let entrypoint = mod_artifact_dir("vencord").join("patcher.js");
    /// // Linux: /home/CoolPerson/.cache/electron-hook/mods/vencord/patcher.js
    /// // Windows: C:/Users/CoolPerson/AppData/Local/electron-hook/mods/vencord/patcher.js
    /// // MacOS: TODO
    /// ```
    pub mod_entrypoint: String,

    /// An optional alternative profile for the mod.
    ///
    /// A profile is a unique instance of an application's data directory - meaning separate settings, cache, chromium instance, etc.
    /// You do not need to use this for basic installs, but if you want to run multiple instances of the same client with different mods or settings, you can use this.
    ///
    /// Preferably, you should get the path using [electron_hook::paths::data_profile_dir]
    ///
    /// You can use it like so:
    ///
    /// ```rust
    /// use electron_hook::paths::data_profile_dir;
    /// let profile_dir = data_profile_dir("moonlight");
    /// // Linux: /home/CoolPerson/.local/share/electron-hook/profiles/moonlight
    /// // Windows: C:/Users/CoolPerson/AppData/Roaming/electron-hook/profiles/moonlight
    /// // MacOS: TODO
    /// ```
    pub profile_dir: Option<String>,
}

impl Asar {
    /// Create a new Asar builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the path to the ASAR archive.
    pub fn get_path(&self) -> Option<std::path::PathBuf> {
        (!self.id.is_empty()).then(|| crate::paths::asar_cache_path(&self.id))
    }

    /// Generate a random UUID for the ASAR archive to use.
    #[cfg(feature = "uuid")]
    pub fn with_uuid(mut self) -> Self {
        self.id = uuid::Uuid::new_v4().to_string();
        std::env::set_var("MODLOADER_ASAR_ID", self.id.clone());
        self
    }

    /// Provide a reusable ID for the ASAR archive to use.
    ///
    /// See [Asar::id]
    pub fn with_id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        std::env::set_var("MODLOADER_ASAR_ID", id);
        self
    }

    /// Provide the template for your index.js to use
    ///
    /// See [Asar::template]
    pub fn with_template(mut self, template: &str) -> Self {
        self.template = template.to_string();
        self
    }

    /// Provide the entrypoint for your mod.
    ///
    /// See [Asar::mod_entrypoint]
    pub fn with_mod_entrypoint(mut self, mod_entrypoint: &str) -> Self {
        self.mod_entrypoint = mod_entrypoint.to_string();
        std::env::set_var("MODLOADER_MOD_ENTRYPOINT", mod_entrypoint);
        self
    }

    /// Provide the WM_CLASS of the application on launch.
    ///
    /// See [Asar::wm_class]
    pub fn with_wm_class(mut self, wm_class: &str) -> Self {
        self.wm_class = Some(wm_class.to_string());
        std::env::set_var("MODLOADER_WM_CLASS", wm_class);
        self
    }

    /// Provide the profile directory for your mod.
    ///
    /// See [Asar::profile_dir]
    pub fn with_profile_dir(mut self, profile_dir: &str) -> Self {
        self.profile_dir = Some(profile_dir.to_string());
        std::env::set_var("MODLOADER_PROFILE_DIR", profile_dir);
        self
    }

    /// Create the ASAR file and write it to disk, returning the path to the ASAR file.
    ///
    /// See [Usage](crate::asar::Asar#usage) for how the path is generated.
    pub fn create(&self) -> Result<std::path::PathBuf, String> {
        use crate::paths::asar_cache_path;

        let asar_path = asar_cache_path(&self.id);

        let mut asar = asar::AsarWriter::new();

        asar.write_file("index.js", self.template.clone(), false)
            .map_err(|e| format!("Failed to write index.js: {e}"))?;

        asar.write_file("package.json", make_package_json(&self.wm_class), false)
            .map_err(|e| format!("Failed to write package.json: {e}"))?;

        let file = std::fs::File::create(&asar_path)
            .map_err(|e| format!("Failed to create file at {}: {e}", asar_path.display()))?;

        asar.finalize(file)
            .map_err(|e| format!("Failed to write asar to disk with error: {e}"))?;

        Ok(asar_path)
    }
}
