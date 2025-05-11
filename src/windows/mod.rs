mod hooks;

use detours_sys::{DetourCreateProcessWithDllExA, _PROCESS_INFORMATION, _STARTUPINFOA};
use winapi::um::{
    handleapi::CloseHandle,
    processthreadsapi::ResumeThread,
    winbase::CREATE_SUSPENDED,
    winuser::{MessageBoxA, MB_ICONERROR},
};

pub fn launch(
    executable: &str,
    library_path: &str,
    asar_path: &str,
    args: Vec<String>,
) -> Result<Option<u32>, String> {
    unsafe {
        let process_args = std::env::args().skip(1).collect::<Vec<String>>();
        let process_args_json =
            serde_json::to_string(&process_args).unwrap_or_else(|_| "[]".into());

        let executable = std::path::Path::new(executable);

        let working_dir = executable
            .parent()
            .ok_or("Failed to get executable directory".to_string())?;

        let shared_object = std::ffi::CString::new(library_path)
            .map_err(|_| "Failed to convert shared object path to CString")?;

        let folder_name = working_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("Failed to get directory name as string")?;

        // Set env vars needed for the child processes
        std::env::set_var("MODLOADER_ASAR_PATH", asar_path);
        std::env::set_var("MODLOADER_EXECUTABLE", std::env::current_exe().unwrap());
        std::env::set_var("MODLOADER_LIBRARY_PATH", library_path);
        std::env::set_var("MODLOADER_FOLDER_NAME", folder_name);
        std::env::set_var("MODLOADER_ORIGINAL_ASAR_RELATIVE", "../_app.asar");
        std::env::set_var("MODLOADER_PROCESS_ARGV", process_args_json);

        let working_dir = std::ffi::CString::new(working_dir.parent().unwrap().to_str().unwrap())
            .map_err(|_| "Failed to convert directory path to CString")?;

        let executable_path = executable
            .to_str()
            .ok_or("Failed to convert executable path to string")?;

        let executable = std::ffi::CString::new(executable_path)
            .map_err(|_| "Failed to convert executable path to CString")?;

        let args = args.join(" ");
        let command_line = format!("\"{executable_path}\" {args}");
        let command_line = std::ffi::CString::new(command_line)
            .map_err(|_| "Failed to convert command line to CString")?;

        let mut startup_info: _STARTUPINFOA = std::mem::zeroed();
        let mut process_info: _PROCESS_INFORMATION = std::mem::zeroed();
        let result = DetourCreateProcessWithDllExA(
            executable.as_ptr() as *mut i8,
            command_line.as_ptr() as *mut i8,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            CREATE_SUSPENDED,
            std::ptr::null_mut(),
            working_dir.as_ptr() as *mut i8,
            &raw mut startup_info as _,
            &raw mut process_info as _,
            shared_object.as_ptr() as *mut i8,
            None,
        );

        if result == 0 {
            MessageBoxA(
                std::ptr::null_mut(),
                "Failed to hook CreateProcessW. Please report this issue.".as_ptr() as *const i8,
                "Error Hooking".as_ptr() as *const i8,
                MB_ICONERROR,
            );
            return Err("Failed to hook CreateProcessW. Please report this issue.".into());
        }

        ResumeThread(process_info.hThread as _);

        CloseHandle(process_info.hThread as _);
        CloseHandle(process_info.hProcess as _);
    }

    Ok(None)
}
