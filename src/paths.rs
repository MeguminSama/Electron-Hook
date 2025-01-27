//! Path utilities

fn cache_dir() -> std::path::PathBuf {
    dirs::cache_dir()
        .expect("Failed to get cache directory")
        .join("electron-hook")
}

fn asar_cache_dir() -> std::path::PathBuf {
    ensure_dir(cache_dir().join("asar"))
}

/// The path to a specific .asar file
pub fn asar_cache_path(asar_id: &str) -> std::path::PathBuf {
    asar_cache_dir().join(format!("{asar_id}.asar"))
}

fn mod_artifacts_dir() -> std::path::PathBuf {
    ensure_dir(cache_dir().join("mods"))
}

/// The path to a specific mod artifact folder
pub fn mod_artifact_dir(mod_name: &str) -> std::path::PathBuf {
    mod_artifacts_dir().join(mod_name)
}

fn data_dir() -> std::path::PathBuf {
    dirs::data_dir()
        .expect("Failed to get data directory")
        .join("electron-hook")
}

fn data_profiles_dir() -> std::path::PathBuf {
    ensure_dir(data_dir().join("profiles"))
}

/// The path to a specific profile directory
pub fn data_profile_dir(profile_id: &str) -> std::path::PathBuf {
    data_profiles_dir().join(profile_id)
}

/// Ensure a directory exists, recursively creating it if it doesn't
pub fn ensure_dir(path: std::path::PathBuf) -> std::path::PathBuf {
    if !path.exists() {
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create directory: {e}"))
            .unwrap();
    }
    path
}
