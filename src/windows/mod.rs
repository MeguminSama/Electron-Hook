mod hooks;

use detours_sys::{DetourCreateProcessWithDllExA, _PROCESS_INFORMATION, _STARTUPINFOA};
use winapi::um::{
    handleapi::CloseHandle,
    processthreadsapi::ResumeThread,
    winbase::CREATE_SUSPENDED,
    winuser::{MessageBoxA, MB_ICONERROR},
};

pub fn launch(
    directory: &str,
    shared_object_path: &str,
    asar_path: &str,
    args: Vec<String>,
) -> Result<std::process::ExitStatus, String> {
    let directory = std::path::Path::new(directory);

    let executable = get_latest_executable(directory)?;

    let working_dir = executable
        .parent()
        .ok_or("Failed to get executable directory".to_string())?;

    let shared_object = std::ffi::CString::new(shared_object_path) //.replace(".exe", ".dll"))
        .map_err(|_| "Failed to convert shared object path to CString")?;

    let working_dir = std::ffi::CString::new(working_dir.parent().unwrap().to_str().unwrap())
        .map_err(|_| "Failed to convert directory path to CString")?;

    let mut process_info: _PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    let mut startup_info: _STARTUPINFOA = unsafe { std::mem::zeroed() };

    // On Windows, you're supposed to provide both lpApplicationName and lpCommandLine even though lpApplicationName is redundant...
    let executable = std::ffi::CString::new(executable.to_str().unwrap())
        .map_err(|_| "Failed to convert executable path to CString")?;

    let args = args.join(" ");
    let command_line =
        std::ffi::CString::new(format!("\"{}\" {}", executable.to_str().unwrap(), args))
            .map_err(|_| "Failed to convert command line to CString")?;

    // Set env vars needed for the child processes
    std::env::set_var("MODLOADER_ASAR_PATH", asar_path);
    std::env::set_var("MODLOADER_DLL_PATH", shared_object_path);

    let result = unsafe {
        DetourCreateProcessWithDllExA(
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
        )
    };

    if result == 0 {
        unsafe {
            MessageBoxA(
                std::ptr::null_mut(),
                "Failed to hook CreateProcessW. Please report this issue.".as_ptr() as *const i8,
                "Error Hooking".as_ptr() as *const i8,
                MB_ICONERROR,
            );
            return Err("Failed to hook CreateProcessW. Please report this issue.".into());
        }
    }

    unsafe {
        ResumeThread(process_info.hThread as _);

        CloseHandle(process_info.hThread as _);
        CloseHandle(process_info.hProcess as _);
    }

    Ok(std::process::ExitStatus::default())
}

fn get_latest_executable(dir: &std::path::Path) -> Result<std::path::PathBuf, String> {
    let dir_name = dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Failed to get directory name as string")?;

    let target_exe_name = format!("{}.exe", dir_name);

    let files = std::fs::read_dir(dir)
        .map_err(|_| "Failed to read directory")?
        .flatten()
        .collect::<Vec<_>>();

    if !files.iter().any(|f| f.file_name() == "Update.exe") {
        return Err("The provided directory does not contain Update.exe.".into());
    }

    let mut app_dirs: Vec<_> = files
        .iter()
        .filter_map(|f| f.file_name().to_str().map(|s| s.to_string()))
        .filter(|f| f.starts_with("app-"))
        .collect();

    app_dirs.sort_by(|a, b| {
        let parse_version = |s: &str| -> Result<Vec<u32>, ()> {
            // Split into prefix and version parts
            let version_str = s.split_once('-').map(|x| x.1).ok_or(())?;
            // Parse each numeric component
            version_str
                .split('.')
                .map(|num| num.parse().map_err(|_| ()))
                .collect()
        };

        match (parse_version(a), parse_version(b)) {
            (Ok(a_ver), Ok(b_ver)) => b_ver.cmp(&a_ver), // Both valid: compare versions
            (Ok(_), Err(_)) => std::cmp::Ordering::Less, // Valid < Invalid
            (Err(_), Ok(_)) => std::cmp::Ordering::Greater, // Invalid > Valid
            (Err(_), Err(_)) => std::cmp::Ordering::Equal, // Invalid entries stay at the end
        }
    });

    for app in app_dirs {
        let app_dir = dir.join(app);

        let Ok(app_files) = std::fs::read_dir(&app_dir) else {
            continue;
        };

        let app_files = app_files.flatten().collect::<Vec<_>>();

        if app_files.iter().any(|f| *f.file_name() == *target_exe_name) {
            return Ok(app_dir.join(target_exe_name));
        }
    }

    Err("Failed to find a valid Discord executable".into())
}
