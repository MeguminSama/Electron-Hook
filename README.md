# electron-hook

A Rust library for loading mods into Electron applications without patching any files.

[![](https://img.shields.io/static/v1?label=Sponsor&message=%E2%9D%A4&logo=GitHub&color=%23fe8e86)](https://github.com/sponsors/MeguminSama)

This project was designed to ease some pain points with modding Discord, but it can be used for most Electron applications.

For some real-life uses of electron-hook, check out:

- [moonlight launcher](https://github.com/meguminsama/moonlight-launcher)
- [Vencord Launcher](https://github.com/meguminsama/vencord-launcher)

# Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
electron-hook = "0.2.0"

[lib]
crate-type = ["cdylib"]
```

And in your `lib.rs`:

```rust
pub use electron_hook::*;
```

When you build your project with `--lib` it will generate a `.dll` or `.so`, which you can pass the path of into `electron_hook::launch`

# Usage

For a better example, check out the [Documentation](https://docs.rs/electron-hook)

```rust
electron_hook::launch(&electron_executable, &library_path, &asar_path, vec![], true);
```
