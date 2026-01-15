use std::env;
use std::io::{self, Write};
use std::process::Command;

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::path::Path;

const BUILTINS: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];

enum InputCommand {
    Cd { path: String },
    Echo { input: String },
    Executable { program: String, args: String },
    Exit,
    Pwd,
    Type { input: String },
    Unknown,
}

impl From<Vec<String>> for InputCommand {
    fn from(value: Vec<String>) -> Self {
        let cmd = value.iter().next().unwrap();

        match cmd {
            val if val.eq("exit") => Self::Exit,
            val if val.starts_with("cd") => Self::Cd {
                path: val[2..].trim().to_string(),
            },
            val if val.starts_with("echo") => Self::Echo {
                input: val[4..].trim().to_string(),
            },
            val if val.starts_with("type") => Self::Type {
                input: value.iter().next().unwrap().to_owned(),
            },
            val if val.starts_with("pwd") => Self::Pwd,
            val if is_executable(val.split(" ").next().unwrap(), false) => {
                let mut input = val.split(" ");
                let program = input.next().unwrap();
                let args = &val[program.len()..].trim();

                Self::Executable {
                    program: String::from(program),
                    args: args.to_string(),
                }
            }
            _ => Self::Unknown,
        }
    }
}

#[cfg(unix)]
fn is_unix_executable(path: &Path) -> anyhow::Result<bool> {
    use libc::{X_OK, access};

    let cstr = std::ffi::CString::new(path.as_os_str().as_bytes().to_vec())?;
    let res = unsafe { access(cstr.as_ptr(), X_OK) };
    Ok(res == 0)
}

fn is_executable(input: &str, should_print: bool) -> bool {
    let path = std::env::var("PATH").unwrap();
    let path_entries = env::split_paths(&path);

    let mut found = false;

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
                match is_unix_executable(&candidate) {
                    Ok(true) => {
                        if should_print {
                            println!("{} is {}", input, candidate.display());
                        }
                        found = true;
                        break;
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
                    if should_print {
                        println!("{} is {}", input, candidate.display());
                    }
                    found = true;
                    break;
                }
            }
        }
    }

    found
}

fn get_type(input: String) {
    if BUILTINS.contains(&&*input) {
        println!("{} is a shell builtin", input);
        return;
    }

    if !is_executable(&input, true) {
        println!("{}: not found", input)
    }
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        let input_split = match shlex::split(input) {
            Some(val) => val,
            None => return,
        };

        if input_split.is_empty() {
            continue;
        }

        let command_str = input_split.iter().next().unwrap();
        if command_str.is_empty() {
            continue;
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
            InputCommand::Echo { input } => println!("{}", input),
            InputCommand::Type { input } => get_type(input),
            InputCommand::Pwd => println!("{}", std::env::current_dir().unwrap().display()),
            InputCommand::Executable { program, args } => {
                let output = Command::new(program)
                    .args(args.split(" "))
                    .output()
                    .expect("Failed to execute process");

                if !output.stdout.is_empty() {
                    print!("{}", String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    eprint!("{}", String::from_utf8_lossy(&output.stderr));
                }
            }
            InputCommand::Unknown => {
                eprintln!("{}: command not found", input.split(" ").next().unwrap())
            }
        }
    }
}
