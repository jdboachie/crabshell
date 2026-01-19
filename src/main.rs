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

// const REDIRECTION_FLAGS: [&str; 6] = [">", ">>", "1>", "1>>", "2>", "2>>"];

enum Redirection {
    StdoutWrite { path: String },
    StderrWrite { path: String },
    StdoutAppend { path: String },
    StderrAppend { path: String },
}

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
                path: values_iter.next().unwrap_or_default(),
            },
            "echo" => Self::Echo {
                input: values_iter.collect::<Vec<String>>().join(" "),
            },
            "type" => Self::Type {
                input: values_iter.next().unwrap_or_default().to_string(),
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

fn print_or_write(
    std_out_str: Option<String>,
    std_err_str: Option<String>,
    redirection: Option<Redirection>,
) {
    if let Some(redirect) = redirection {
        match redirect {
            Redirection::StdoutWrite { path } => {
                let _ = std::fs::write(path, std_out_str.unwrap_or_default().trim());
                if let Some(err) = std_err_str {
                    println!("{}", err.trim());
                }
            }
            Redirection::StdoutAppend { path } => {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    let _ = std::fs::write(
                        path,
                        contents + "\n" + std_out_str.unwrap_or_default().as_str(),
                    );
                } else {
                    let _ = std::fs::write(path, std_out_str.unwrap_or_default());
                };
                if let Some(err) = std_err_str {
                    println!("{}", err.trim());
                }
            }
            Redirection::StderrWrite { path } => {
                let _ = std::fs::write(path, std_err_str.unwrap_or_default().trim());
                if let Some(out) = std_out_str {
                    println!("{}", out.trim());
                }
            }
            Redirection::StderrAppend { path } => {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    let _ = std::fs::write(
                        path,
                        contents + "\n" + std_err_str.unwrap_or_default().as_str(),
                    );
                } else {
                    let _ = std::fs::write(path, std_err_str.unwrap_or_default().as_str());
                }
                if let Some(out) = std_out_str {
                    println!("{}", out.trim());
                }
            }
        }
    } else {
        if let Some(out) = std_out_str {
            println!("{}", out.trim());
        }
        if let Some(err) = std_err_str {
            eprintln!("{}", err.trim());
        }
    }
}

fn check_extract_redirection(input_split: &mut Vec<String>) -> Option<Redirection> {
    let maybe_pos = input_split
        .clone()
        .into_iter()
        .position(|s| s == ">" || s == ">>" || s == "1>" || s == "1>>" || s == "2>" || s == "2>>");

    if let Some(pos) = maybe_pos {
        let res: Option<Redirection>;
        let out_path = Some(input_split[pos + 1].clone()).unwrap(); // handle index outof bounds error here

        match &**input_split.get(pos).unwrap() {
            ">" | "1>" => res = Some(Redirection::StdoutWrite { path: out_path }),
            ">>" | "1>>" => res = Some(Redirection::StdoutAppend { path: out_path }),
            "2>" => res = Some(Redirection::StderrWrite { path: out_path }),
            "2>>" => res = Some(Redirection::StderrAppend { path: out_path }),
            _ => res = None,
        }

        input_split.remove(pos + 1);
        input_split.remove(pos);

        return res;
    }

    None
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

        // let maybe_redirect_pos = input_split
        //     .clone()
        //     .into_iter()
        //     .position(|s| s == ">" || s == "1>");

        // let mut out_path: Option<String> = None;
        // if let Some(pos) = maybe_redirect_pos {
        //     out_path = Some(input_split[pos + 1].clone()); // handle index outof bounds error here
        //     input_split.remove(pos + 1);
        //     input_split.remove(pos);
        // }

        let redirection = check_extract_redirection(&mut input_split);

        let command = InputCommand::from(input_split);
        match command {
            InputCommand::Cd { path } => {
                if path == "~" || path == "#" {
                    #[cfg(windows)]
                    {
                        std::env::set_current_dir(std::env::var("USERPROFILE").unwrap())
                            .unwrap_or_else(|e| eprintln!("cd: {}: {}", path, e));
                    }

                    #[cfg(unix)]
                    {
                        std::env::set_current_dir(std::env::var("HOME").unwrap())
                            .unwrap_or_else(|e| eprintln!("cd: {}: {}", path, e));
                    }
                } else {
                    std::env::set_current_dir(&path)
                        .unwrap_or_else(|_| eprintln!("cd: {}: No such file or directory", &path));
                }
            }
            InputCommand::Exit => break,
            InputCommand::Echo { input } => {
                print_or_write(Some(input), None, redirection);
            }
            InputCommand::Type { input } => {
                let cmd_type = get_type(&input);

                let out_str = match cmd_type {
                    CommandType::Builtin => format!("{} is a shell builtin", input),
                    CommandType::Executable { path } => format!("{} is {}", input, path),
                    CommandType::Unknown => format!("{}: not found", input),
                };

                print_or_write(Some(out_str), None, redirection);
            }
            InputCommand::Pwd => {
                print_or_write(
                    Some(std::env::current_dir().unwrap().display().to_string()),
                    None,
                    redirection,
                );
            }
            InputCommand::Executable { program, args } => {
                let output = Command::new(program)
                    .args(args)
                    .output()
                    .expect("Failed to execute process");

                let std_out_str = (!output.stdout.is_empty())
                    .then(|| String::from_utf8_lossy(&output.stdout).to_string());
                let std_err_str = (!output.stderr.is_empty())
                    .then(|| String::from_utf8_lossy(&output.stderr).to_string());

                print_or_write(std_out_str, std_err_str, redirection)
            }
            InputCommand::Unknown => {
                eprintln!("{}: command not found", input.split(" ").next().unwrap())
            }
        }

        print!("$ ");
        io::stdout().flush().unwrap();
    }
}
