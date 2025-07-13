use std::{
    ffi::{c_char, c_void, CStr, CString},
    mem::transmute,
    str::FromStr,
};

use detours_sys::{
    DetourAttach, DetourCreateProcessWithDllW, DetourIsHelperProcess, DetourRestoreAfterWith,
    DetourTransactionAbort, DetourTransactionBegin, DetourTransactionCommit, DetourUpdateThread,
};
use widestring::U16CString;
use winapi::{
    shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID},
    um::{
        libloaderapi::{GetModuleHandleA, GetProcAddress},
        minwinbase::LPSECURITY_ATTRIBUTES,
        processthreadsapi::{
            GetCurrentThread, ResumeThread, LPPROCESS_INFORMATION, LPSTARTUPINFOW,
        },
        winnt::{DLL_PROCESS_ATTACH, HANDLE, LPCWSTR, LPWSTR},
        winuser::MessageBoxA,
    },
};

#[cfg(debug_assertions)]
#[link(name = "kernel32")]
unsafe extern "system" {
    unsafe fn AllocConsole() -> BOOL;
}

// Environment variables
mod env {
    use std::sync::LazyLock;

    macro_rules! lazy_env {
        ($name:expr) => {
            LazyLock::new(|| std::env::var($name).unwrap())
        };
    }

    pub static ASAR_PATH: LazyLock<String> = lazy_env!("MODLOADER_ASAR_PATH");
    pub static EXE_PATH: LazyLock<String> = lazy_env!("MODLOADER_EXECUTABLE");
    pub static DLL_PATH: LazyLock<String> = lazy_env!("MODLOADER_LIBRARY_PATH");
    pub static FOLDER_NAME: LazyLock<String> = lazy_env!("MODLOADER_FOLDER_NAME");
}

// Import the original functions to be hooked
#[allow(non_upper_case_globals)]
mod original {
    use winapi::um::{
        fileapi::{CreateFileW as CreateFileW_, GetFileAttributesW as GetFileAttributesW_},
        processthreadsapi::CreateProcessW as CreateProcessW_,
        winbase::MoveFileExW as MoveFileExW_,
    };

    #[link(name = "user32")]
    unsafe extern "C" {
        #[link_name = "SetCurrentProcessExplicitAppUserModelID"]
        unsafe fn SetAUMID_(app_id: *const u16);
    }

    type FnPtr = *mut std::ffi::c_void;

    pub static mut GetFileAttributesW: FnPtr = GetFileAttributesW_ as _;
    pub static mut CreateFileW: FnPtr = CreateFileW_ as _;
    pub static mut CreateProcessW: FnPtr = CreateProcessW_ as _;
    pub static mut MoveFileExW: FnPtr = MoveFileExW_ as _;
    pub static mut SetAUMID: FnPtr = SetAUMID_ as _;
    pub static mut uv_fs_lstat: FnPtr = std::ptr::null_mut();
}

// We need to make sure that our hooks only affect the current version of Discord.
// Otherwise, the updater might not work!
fn prefix_file(file_name: &str) -> String {
    format!("{}\\{}", env::FOLDER_NAME.as_str(), file_name)
}

macro_rules! error_hooking_msg {
    ($msg:expr) => {
        MessageBoxA(
            std::ptr::null_mut(),
            $msg.as_ptr() as *const i8,
            "Error Hooking".as_ptr() as *const i8,
            0,
        );
    };
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    _hinst_dll: HINSTANCE,
    fwd_reason: DWORD,
    _lpv_reserved: LPVOID,
) -> i32 {
    if DetourIsHelperProcess() == 1 {
        return 1;
    }

    if fwd_reason != DLL_PROCESS_ATTACH {
        return 1;
    }

    DetourRestoreAfterWith();

    DetourTransactionBegin();
    DetourUpdateThread(GetCurrentThread() as _);

    #[cfg(debug_assertions)]
    AllocConsole();

    macro_rules! attach {
        ($orig:ident, $target:ident) => {
            let result = DetourAttach(&raw mut original::$orig, $target as _);

            if result != 0 {
                error_hooking_msg!(format!(
                    "Failed to hook {}. Please report this issue.",
                    stringify!($orig)
                ));
                DetourTransactionAbort();
                return 1;
            };
        };
    }

    attach!(GetFileAttributesW, get_file_attributes_w);
    attach!(CreateFileW, create_file_w);
    attach!(MoveFileExW, move_file_ex_w);
    attach!(CreateProcessW, create_process_w);
    attach!(SetAUMID, set_aumid);

    fn get_executable_name() -> Option<CString> {
        let current_exe = std::env::current_exe().ok()?;
        let file_name = current_exe.file_name()?;
        let file_name = file_name.to_str()?;
        let file_name_cstr = CString::new(file_name).ok()?;
        Some(file_name_cstr)
    }

    if let Some(current_exe) = get_executable_name() {
        let process_handle = GetModuleHandleA(current_exe.as_ptr());
        if !process_handle.is_null() {
            let uv_fs_lstat_ptr = GetProcAddress(process_handle as _, c"uv_fs_lstat".as_ptr());
            if !uv_fs_lstat_ptr.is_null() {
                original::uv_fs_lstat = uv_fs_lstat_ptr as _;
                attach!(uv_fs_lstat, uv_fs_lstat);
            }
        }
    }

    DetourTransactionCommit();

    1
}

type UvFsLstat = unsafe extern "C" fn(
    _loop: *const c_void,
    req: *const c_void,
    path: *const c_char,
    cb: *const c_void,
) -> i32;
unsafe extern "C" fn uv_fs_lstat(
    _loop: *const c_void,
    _req: *const c_void,
    path: *const c_char,
    _cb: *const c_void,
) -> i32 {
    let uv_fs_lstat: UvFsLstat = transmute(original::uv_fs_lstat);

    let file_name = CStr::from_ptr(path as _);
    let file_name = file_name.to_str().unwrap();

    if file_name.contains("resources\\_app.asar") {
        let redirect_to = file_name.replace("\\_app.asar", "\\app.asar");
        let redirect_to_c = std::ffi::CString::new(redirect_to.as_str()).unwrap();

        return uv_fs_lstat(_loop, _req, redirect_to_c.as_ptr() as _, _cb);
    }
    if file_name.contains(&prefix_file("resources\\app.asar")) {
        let asar_path_cstr = std::ffi::CString::new(env::ASAR_PATH.as_str()).unwrap();

        return uv_fs_lstat(_loop, _req, asar_path_cstr.as_ptr() as _, _cb);
    }
    uv_fs_lstat(_loop, _req, path, _cb)
}

type GetFileAttributesW = unsafe extern "C" fn(LPCWSTR) -> DWORD;

unsafe extern "C" fn get_file_attributes_w(lp_file_name: LPCWSTR) -> DWORD {
    let file_name = U16CString::from_ptr_str(lp_file_name).to_string().unwrap();

    let get_file_attributes_w: GetFileAttributesW = transmute(original::GetFileAttributesW);

    if file_name.contains("resources\\_app.asar") {
        let redirect_to = file_name.replace("\\_app.asar", "\\app.asar");
        let redirect_to_c = std::ffi::CString::new(redirect_to.as_str()).unwrap();
        let redirect_to = U16CString::from_str(redirect_to_c.to_str().unwrap()).unwrap();

        return get_file_attributes_w(redirect_to.as_ptr());
    }

    if file_name.contains(&prefix_file("resources\\app.asar")) {
        let asar_path_cstr = std::ffi::CString::new(env::ASAR_PATH.as_str()).unwrap();
        let asar_path = U16CString::from_str(asar_path_cstr.to_str().unwrap()).unwrap();

        return get_file_attributes_w(asar_path.as_ptr());
    }

    get_file_attributes_w(lp_file_name)
}

type CreateFileW = unsafe extern "C" fn(
    LPCWSTR,
    DWORD,
    DWORD,
    LPSECURITY_ATTRIBUTES,
    DWORD,
    DWORD,
    HANDLE,
) -> HANDLE;

unsafe extern "C" fn create_file_w(
    lp_file_name: LPCWSTR,
    dw_desired_access: DWORD,
    dw_share_mode: DWORD,
    lp_security_attributes: LPSECURITY_ATTRIBUTES,
    dw_creation_disposition: DWORD,
    dw_flags_and_attributes: DWORD,
    h_template_file: HANDLE,
) -> HANDLE {
    let create_file_w: CreateFileW = transmute(original::CreateFileW);

    let file_name = U16CString::from_ptr_str(lp_file_name).to_string().unwrap();

    if file_name.contains("resources\\_app.asar") {
        let redirect_to = file_name.replace("\\_app.asar", "\\app.asar");
        let redirect_to_c = std::ffi::CString::new(redirect_to.as_str()).unwrap();
        let redirect_to = U16CString::from_str(redirect_to_c.to_str().unwrap()).unwrap();

        return create_file_w(
            redirect_to.as_ptr(),
            dw_desired_access,
            dw_share_mode,
            lp_security_attributes,
            dw_creation_disposition,
            dw_flags_and_attributes,
            h_template_file,
        );
    }

    if file_name.contains(&prefix_file("resources\\app.asar")) {
        let asar_path_cstr = std::ffi::CString::new(env::ASAR_PATH.as_str()).unwrap();
        let asar_path = U16CString::from_str(asar_path_cstr.to_str().unwrap()).unwrap();

        return create_file_w(
            asar_path.as_ptr(),
            dw_desired_access,
            dw_share_mode,
            lp_security_attributes,
            dw_creation_disposition,
            dw_flags_and_attributes,
            h_template_file,
        );
    }

    create_file_w(
        lp_file_name,
        dw_desired_access,
        dw_share_mode,
        lp_security_attributes,
        dw_creation_disposition,
        dw_flags_and_attributes,
        h_template_file,
    )
}

type MoveFileExW = unsafe extern "C" fn(
    lp_existing_file_name: LPCWSTR,
    lp_new_file_name: LPCWSTR,
    dw_flags: DWORD,
) -> BOOL;

// This is needed to stop the updater from renaming app.asar to _app.asar
unsafe extern "C" fn move_file_ex_w(
    lp_existing_file_name: LPCWSTR,
    lp_new_file_name: LPCWSTR,
    dw_flags: DWORD,
) -> BOOL {
    let move_file_ex_w: MoveFileExW = transmute(original::MoveFileExW);

    let new_file_name = U16CString::from_ptr_str(lp_new_file_name)
        .to_string()
        .unwrap();

    // MoveFileExW moves _app.asar when we update Discord, so we should update MODLOADER_FOLDER_NAME just in case.
    if new_file_name.contains("\\_app.asar") {
        let redirect_to = new_file_name.replace("\\_app.asar", "\\app.asar");

        if let Some(folder_name) = std::path::Path::new(&redirect_to)
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
        {
            std::env::set_var(
                "MODLOADER_FOLDER_NAME",
                folder_name.to_string_lossy().to_string(),
            );
        }

        let redirect_to_c = std::ffi::CString::new(redirect_to.as_str()).unwrap();
        let redirect_to = U16CString::from_str(redirect_to_c.to_str().unwrap()).unwrap();

        return move_file_ex_w(lp_existing_file_name, redirect_to.as_ptr(), dw_flags);
    }

    move_file_ex_w(lp_existing_file_name, lp_new_file_name, dw_flags)
}

type CreateProcessW = unsafe extern "C" fn(
    lp_application_name: LPCWSTR,
    lp_command_line: LPWSTR,
    lp_process_attributes: LPSECURITY_ATTRIBUTES,
    lp_thread_attributes: LPSECURITY_ATTRIBUTES,
    b_inherit_handles: BOOL,
    dw_creation_flags: DWORD,
    lp_environment: LPVOID,
    lp_current_directory: LPCWSTR,
    lp_startup_info: LPSTARTUPINFOW,
    lp_process_information: LPPROCESS_INFORMATION,
) -> BOOL;

unsafe extern "C" fn create_process_w(
    lp_application_name: LPCWSTR,
    lp_command_line: LPWSTR,
    lp_process_attributes: LPSECURITY_ATTRIBUTES,
    lp_thread_attributes: LPSECURITY_ATTRIBUTES,
    b_inherit_handles: BOOL,
    dw_creation_flags: DWORD,
    lp_environment: LPVOID,
    lp_current_directory: LPCWSTR,
    lp_startup_info: LPSTARTUPINFOW,
    lp_process_information: LPPROCESS_INFORMATION,
) -> BOOL {
    let create_process_w: CreateProcessW = transmute(original::CreateProcessW);

    let command_line = U16CString::from_ptr_str(lp_command_line)
        .to_string()
        .unwrap();

    // When the updater "restarts" Discord, it doesn't seem to pass any arguments to the process.
    // So we can just check if the command contains "--" to make sure we hook the new Discord instance.
    if command_line.contains("--") && !command_line.contains("--type=renderer") {
        // Run the original CreateProcessW
        return create_process_w(
            lp_application_name,
            lp_command_line,
            lp_process_attributes,
            lp_thread_attributes,
            b_inherit_handles,
            dw_creation_flags,
            lp_environment,
            lp_current_directory,
            lp_startup_info,
            lp_process_information,
        );
    }

    let dll_path = CString::from_str(env::DLL_PATH.as_str()).unwrap();

    #[allow(
        clippy::missing_transmute_annotations,
        reason = "Excessive boilerplate"
    )]
    let success = DetourCreateProcessWithDllW(
        lp_application_name,
        lp_command_line,
        lp_process_attributes as _,
        lp_thread_attributes as _,
        b_inherit_handles,
        dw_creation_flags,
        lp_environment as _,
        lp_current_directory,
        lp_startup_info as _,
        lp_process_information as _,
        dll_path.as_ptr(),
        Some(std::mem::transmute(original::CreateProcessW)),
    );

    if success != 1 {
        eprintln!("[Electron Hook] Failed to create process");
        return success;
    }

    ResumeThread((*lp_process_information).hThread as _);

    success
}

type SetAUMID = unsafe extern "system" fn(lp_app_id: LPCWSTR);

unsafe extern "system" fn set_aumid(_lp_app_id: LPCWSTR) {
    let set_aumid: SetAUMID = std::mem::transmute(original::SetAUMID);

    // We set the AUMID to be the path of the launcher exe, so it looks like the launcher in the taskbar.
    // I spent way too long working on this.
    let new_id = U16CString::from_str(env::EXE_PATH.as_str()).unwrap();
    set_aumid(new_id.as_ptr());
}
