use std::env;
use std::io::{self, Write};
use std::process::Command;

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::path::Path;

enum CommandType {
    Builtin,
    Executable { path: String },
    Unknown,
}

const BUILTINS: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];

enum InputCommand {
    Cd { path: String },
    Echo { input: String },
    Executable { program: String, args: Vec<String> },
    Exit,
    Pwd,
    Type { input: String },
    Unknown,
}

impl From<Vec<String>> for InputCommand {
    fn from(value: Vec<String>) -> Self {
        let mut values_iter = value.into_iter();
        let cmd = values_iter.next().unwrap();

        match cmd.as_str() {
            "exit" => Self::Exit,
            "cd" => Self::Cd {
                path: values_iter.next().unwrap(),
            },
            "echo" => Self::Echo {
                input: values_iter.collect::<Vec<String>>().join(" "),
            },
            "type" => Self::Type {
                input: values_iter.next().unwrap().to_string(),
            },
            "pwd" => Self::Pwd,
            _ if find_executable(&cmd).is_some() => Self::Executable {
                program: cmd,
                args: values_iter.collect::<Vec<String>>(),
            },
            _ => Self::Unknown,
        }
    }
}

#[cfg(unix)]
fn find_unix_executable(path: &Path) -> anyhow::Result<bool> {
    use libc::{X_OK, access};

    let cstr = std::ffi::CString::new(path.as_os_str().as_bytes().to_vec())?;
    let res = unsafe { access(cstr.as_ptr(), X_OK) };
    Ok(res == 0)
}

fn find_executable(input: &str) -> Option<String> {
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

fn get_type(input: &String) -> CommandType {
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

fn print_or_write(out_str: &String, out_path: Option<String>) {
    if let Some(path) = out_path {
        let _ = std::fs::write(path, out_str.trim());
    } else {
        println!("{}", out_str);
    }
}

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        let mut input_split = match shlex::split(input) {
            Some(val) => val,
            None => return,
        };

        if input_split.is_empty() {
            continue;
        }

        let maybe_redirect_pos = input_split
            .clone()
            .into_iter()
            .position(|s| s == ">" || s == "1>");

        let mut out_path: Option<String> = None;
        if let Some(pos) = maybe_redirect_pos {
            out_path = Some(input_split[pos + 1].clone()); // handle index outof bounds error here
            input_split.remove(pos + 1);
            input_split.remove(pos);
        }

        let command = InputCommand::from(input_split);
        match command {
            InputCommand::Cd { path } => {
                if path == "~" || path == "#" {
                    #[cfg(windows)]
                    {
                        std::env::set_current_dir(std::env::var("USERPROFILE").unwrap())
                            .unwrap_or_else(|e| println!("cd: {}: {}", path, e));
                        continue;
                    }

                    #[cfg(unix)]
                    {
                        std::env::set_current_dir(std::env::var("HOME").unwrap())
                            .unwrap_or_else(|e| println!("cd: {}: {}", path, e));
                        continue;
                    }
                }

                std::env::set_current_dir(&path)
                    .unwrap_or_else(|_| println!("cd: {}: No such file or directory", &path));
            }
            InputCommand::Exit => break,
            InputCommand::Echo { input } => {
                print_or_write(&input, out_path);
            }
            InputCommand::Type { input } => {
                let cmd_type = get_type(&input);

                let out_str = match cmd_type {
                    CommandType::Builtin => format!("{} is a shell builtin", input),
                    CommandType::Executable { path } => format!("{} is {}", input, path),
                    CommandType::Unknown => format!("{}: not found", input),
                };

                print_or_write(&out_str, out_path);
            }
            InputCommand::Pwd => {
                print_or_write(
                    &std::env::current_dir().unwrap().display().to_string(),
                    out_path,
                );
            }
            InputCommand::Executable { program, args } => {
                let output = Command::new(program)
                    .args(args)
                    .output()
                    .expect("Failed to execute process");

                if !output.stdout.is_empty() {
                    print_or_write(
                        &String::from_utf8_lossy(&output.stdout).to_string(),
                        out_path,
                    );
                }
                if !output.stderr.is_empty() {
                    eprint!("{}", String::from_utf8_lossy(&output.stderr));
                }
            }
            InputCommand::Unknown => {
                eprintln!("{}: command not found", input.split(" ").next().unwrap())
            }
        }

        print!("$ ");
        io::stdout().flush().unwrap();
    }
}
