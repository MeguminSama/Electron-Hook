use std::{
    ffi::{c_char, c_void},
    sync::LazyLock,
};

// Environment variables
mod env {
    use std::sync::LazyLock;

    macro_rules! lazy_env {
        ($name:expr) => {
            LazyLock::new(|| std::env::var($name).unwrap())
        };
    }

    pub static ASAR_PATH: LazyLock<String> = lazy_env!("MODLOADER_ASAR_PATH");
    pub static DLL_PATH: LazyLock<String> = lazy_env!("MODLOADER_DLL_PATH");
    pub static FOLDER_NAME: LazyLock<String> = lazy_env!("MODLOADER_FOLDER_NAME");
    pub static EXE_PATH: LazyLock<String> = lazy_env!("MODLOADER_EXE_PATH");
}

#[repr(C)]
pub struct Interpose {
    pub replacement: *const c_void,
    pub target: *const c_void,
}

unsafe impl Send for Interpose {}
unsafe impl Sync for Interpose {}

macro_rules! dyld_interpose {
    ($name:ident, $target:expr, $replacement:expr) => {
        #[no_mangle]
        #[link_section = "__DATA,__interpose"]
        static $name: Interpose = Interpose {
            replacement: $replacement as _,
            target: $target as _,
        };
    };
}

#[no_mangle]
unsafe extern "C" fn my_fork() -> libc::pid_t {
    let child = libc::fork();
    dbg!("FORKED");
    child
}

dyld_interpose!(FORK, libc::fork, my_fork);
