#![cfg(target_os = "windows")]

//! Bindings to the Microsoft Detours API.
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Rebuild with:
// bindgen ..\build\wrapper.h --allowlist-function "Detour.*" -o bundled_bindings.rs  -- "-fms-compatibility" "-fms-extensions" --target=x86_64-pc-windows-msvc -I ..\ext\detours\src\
include!("bundled_bindings.rs");
