//! Macros for electron-hook

/// Make the application self-executable. **READ THE USAGE FIRST**
///
/// # Warning
///
/// This is dangerous territory, so I'll lay out some ground rules.
///
/// 1. Do not use this macro in executables. Your program **will not work**.
/// 2. This macro is only for shared libraries. Specifically, `cdylib`.
/// 3. This macro is untested with `musl`. Here be dragons.
///
/// # Usage
///
/// First of all, you'll want to mark your application as a shared library.
///
/// Firstly, rename your `main.rs` to `lib.rs`.
///
/// Next up, you need to change your `fn main()`:
///
/// Before:
/// ```rust,ignore
/// fn main() {
/// ```
///
/// After:
/// ```rust,ignore
/// #[no_mangle]
/// fn main() {
/// ```
///
/// Make sure to add this to your `Cargo.toml`:
///
/// ```toml
/// [lib]
/// crate-type = ["cdylib"]
/// ```
///
/// Now, at the top of your `lib.rs`, add this:
///
/// ```rust,ignore
/// electron_hook::make_shared_executable!();
/// ```
///
/// Now, create a new file called `main.rs` (yes, I know...) and add this:
/// ```rust
/// #![no_main]
/// pub use your_crate_name::*;
/// ```
///
/// TODO: Improve the workflow.
///
/// On linux, you'll need to run `cargo build --lib` (with `--release` for releases) to build the shared library.
/// The subsequent `.so` file will be executable.
///
/// On windows, you'll want to build the binary instead, with `cargo build --bin your-crate-name` (with `--release` for releases).
/// The executable will act as a DLL. No extra work is needed.
///
#[cfg(any(doc, all(feature = "self-executable", target_os = "linux")))]
#[allow(clippy::crate_in_macro_def)]
#[macro_export]
macro_rules! make_shared_executable {
    () => {
        mod __shared_executable {
            macro_rules! interp {
                () => {
                    b"/lib64/ld-linux-x86-64.so.2\0"
                };
            }

            #[link_section = ".interp"]
            #[no_mangle]
            pub static INTERP: [u8; interp!().len()] = *interp!();
        }
    };
}
