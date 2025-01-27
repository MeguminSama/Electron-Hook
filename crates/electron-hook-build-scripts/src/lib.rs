/// Apply cargo build scripts required for electron-hook's shared-executable feature.
pub fn build() {
    // Unfortunately, it's unsafe to use rustc-cdylib-link-args transitively in child dependencies
    // So the user must add this to their build script.
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-cdylib-link-arg=/usr/lib/crt1.o");
}
