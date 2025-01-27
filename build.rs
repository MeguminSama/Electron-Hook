fn main() {
    // Apparently we just need to link crt1.o to implement _start
    // along with adding the .interp section in lib.rs

    // We use rustc-cdylib-link-arg because it passes to the dependent crate.
    // whereas rustc-link-arg only applies to the current crate.

    // TODO: This could break in the future...
    // https://github.com/rust-lang/cargo/issues/9562
    // Another repo affected by this: https://github.com/slint-ui/slint/issues/566
    #[cfg(all(feature = "self-executable", target_os = "linux"))]
    println!("cargo:rustc-cdylib-link-arg=/usr/lib/crt1.o");
}
