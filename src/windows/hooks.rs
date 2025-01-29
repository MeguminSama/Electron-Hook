use std::{
    ffi::CString,
    str::FromStr,
    sync::{LazyLock, Mutex},
};

use detours_sys::{
    DetourAttach, DetourCreateProcessWithDllW, DetourIsHelperProcess, DetourRestoreAfterWith,
    DetourTransactionAbort, DetourTransactionBegin, DetourTransactionCommit, DetourUpdateThread,
};
use widestring::U16CString;
use winapi::{
    shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID},
    um::{
        fileapi::{CreateFileW, GetFileAttributesW},
        minwinbase::LPSECURITY_ATTRIBUTES,
        processthreadsapi::{
            CreateProcessW, GetCurrentThread, ResumeThread, LPPROCESS_INFORMATION, LPSTARTUPINFOW,
        },
        winbase::MoveFileExW,
        winnt::{DLL_PROCESS_ATTACH, HANDLE, LPCWSTR, LPWSTR, PVOID},
        winuser::MessageBoxA,
    },
};

static MODLOADER_ASAR_PATH: LazyLock<String> =
    LazyLock::new(|| std::env::var("MODLOADER_ASAR_PATH").unwrap());

static MODLOADER_DLL_PATH: LazyLock<String> =
    LazyLock::new(|| std::env::var("MODLOADER_DLL_PATH").unwrap());

static MODLOADER_FOLDER_NAME: LazyLock<String> =
    LazyLock::new(|| std::env::var("MODLOADER_FOLDER_NAME").unwrap());

// We need to make sure that our hooks only affect the current version of Discord.
// Otherwise, the updater might not work!
fn prefix_file(file_name: &str) -> String {
    format!("{}\\{}", *MODLOADER_FOLDER_NAME, file_name)
}

static mut ORIGINAL_GET_FILE_ATTRIBUTES_W: PVOID = GetFileAttributesW as _;
static mut ORIGINAL_CREATE_FILE_W: PVOID = CreateFileW as _;
static mut ORIGINAL_CREATE_PROCESS_W: PVOID = CreateProcessW as _;
static mut ORIGINAL_MOVE_FILE_EX_W: PVOID = MoveFileExW as _;

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
pub unsafe extern "stdcall" fn DllMain(
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

    let result = DetourAttach(
        &raw mut ORIGINAL_GET_FILE_ATTRIBUTES_W as _,
        get_file_attributes_w as _,
    );

    if result != 0 {
        error_hooking_msg!("Failed to hook GetFileAttributesW. Please report this issue.");
        DetourTransactionAbort();
        return 1;
    }

    let result = DetourAttach(&raw mut ORIGINAL_CREATE_FILE_W as _, create_file_w as _);

    if result != 0 {
        error_hooking_msg!("Failed to hook CreateFileW. Please report this issue.");
        DetourTransactionAbort();
        return 1;
    }

    let result = DetourAttach(&raw mut ORIGINAL_MOVE_FILE_EX_W as _, move_file_ex_w as _);

    if result != 0 {
        error_hooking_msg!("Failed to hook MoveFileExW. Please report this issue.");
        DetourTransactionAbort();
        return 1;
    }

    let result = DetourAttach(
        &raw mut ORIGINAL_CREATE_PROCESS_W as _,
        create_process_w as _,
    );

    if result != 0 {
        error_hooking_msg!("Failed to hook CreateProcessW. Please report this issue on GitHub.");
        DetourTransactionAbort();
        return 1;
    }

    DetourTransactionCommit();

    1
}

unsafe extern "C" fn get_file_attributes_w(lp_file_name: LPCWSTR) -> DWORD {
    let file_name = U16CString::from_ptr_str(lp_file_name).to_string().unwrap();

    let get_file_attributes_w: extern "C" fn(LPCWSTR) -> DWORD =
        std::mem::transmute(ORIGINAL_GET_FILE_ATTRIBUTES_W);

    if file_name.contains("resources\\_app.asar") {
        let redirect_to = file_name.replace("\\_app.asar", "\\app.asar");
        let redirect_to_c = std::ffi::CString::new(redirect_to.as_str()).unwrap();
        let redirect_to = U16CString::from_str(redirect_to_c.to_str().unwrap()).unwrap();

        return get_file_attributes_w(redirect_to.as_ptr());
    }

    if file_name.contains(&prefix_file("resources\\app.asar")) {
        let asar_path_cstr = std::ffi::CString::new(MODLOADER_ASAR_PATH.as_str()).unwrap();
        let asar_path = U16CString::from_str(asar_path_cstr.to_str().unwrap()).unwrap();

        return get_file_attributes_w(asar_path.as_ptr());
    }

    get_file_attributes_w(lp_file_name)
}

unsafe extern "C" fn create_file_w(
    lp_file_name: LPCWSTR,
    dw_desired_access: DWORD,
    dw_share_mode: DWORD,
    lp_security_attributes: LPSECURITY_ATTRIBUTES,
    dw_creation_disposition: DWORD,
    dw_flags_and_attributes: DWORD,
    h_template_file: HANDLE,
) -> HANDLE {
    let file_name = U16CString::from_ptr_str(lp_file_name).to_string().unwrap();

    let create_file_w: extern "C" fn(
        lp_file_name: LPCWSTR,
        dw_desired_access: DWORD,
        dw_share_mode: DWORD,
        lp_security_attributes: LPSECURITY_ATTRIBUTES,
        dw_creation_disposition: DWORD,
        dw_flags_and_attributes: DWORD,
        h_template_file: HANDLE,
    ) -> HANDLE = std::mem::transmute(ORIGINAL_CREATE_FILE_W);
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
        let asar_path_cstr = std::ffi::CString::new(MODLOADER_ASAR_PATH.as_str()).unwrap();
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

type FnMoveFileExW = unsafe extern "C" fn(
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
    let move_file_ex_w: FnMoveFileExW = std::mem::transmute(ORIGINAL_MOVE_FILE_EX_W);

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

type FnCreateProcessW = unsafe extern "C" fn(
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
    let create_process_w: FnCreateProcessW = std::mem::transmute(ORIGINAL_CREATE_PROCESS_W);

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

    let dll_path = CString::from_str(&MODLOADER_DLL_PATH).unwrap();

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
        Some(std::mem::transmute(ORIGINAL_CREATE_PROCESS_W)),
    );

    if success != 1 {
        eprintln!("[Electron Hook] Failed to create process");
        return success;
    }

    ResumeThread((*lp_process_information).hThread as _);

    success
}
