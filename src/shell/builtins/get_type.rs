use std::env;

const BUILTINS: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];

#[cfg(unix)]
fn find_unix_executable(path: &Path) -> anyhow::Result<bool> {
    use libc::{X_OK, access};

    let cstr = std::ffi::CString::new(path.as_os_str().as_bytes().to_vec())?;
    let res = unsafe { access(cstr.as_ptr(), X_OK) };
    Ok(res == 0)
}

pub fn find_executable(input: &str) -> Option<String> {
    let path = std::env::var("PATH").unwrap();
    let path_entries = env::split_paths(&path);

    #[cfg(windows)]
    let path_exts: Vec<String> = env::var("PATHEXT")
        .unwrap_or(".EXE;.CMD;.BAT;.COM".to_string())
        .split(';')
        .map(|s| s.to_lowercase())
        .collect();

    for dir in path_entries {
        if !dir.is_dir() {
            continue;
        }

        #[cfg(unix)]
        {
            let candidate = dir.join(&input);
            if candidate.is_file() {
                match find_unix_executable(&candidate) {
                    Ok(true) => {
                        return Some(candidate.display().to_string());
                    }
                    _ => {}
                }
            }
        }

        #[cfg(windows)]
        {
            for ext in &path_exts {
                let candidate = dir.join(format!("{}{}", input, ext));
                if candidate.is_file() {
                    return Some(candidate.display().to_string());
                }
            }
        }
    }

    None
}

pub enum CommandType {
    Builtin,
    Executable { path: String },
    Unknown,
}

pub fn get_type(input: &String) -> CommandType {
    if BUILTINS.contains(&input.as_str()) {
        CommandType::Builtin
    } else {
        if let Some(path) = find_executable(&input) {
            CommandType::Executable { path }
        } else {
            CommandType::Unknown
        }
    }
}
