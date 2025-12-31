use std::io::{self, Write};
use std::{env};

const BUILTINS: [&str; 3] = ["echo", "exit", "type"];

enum Command {
    Exit,
    Type { input: String },
    Echo { input: String },
    Unknown,
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value {
            "exit" => Self::Exit,
            val if val.starts_with("echo") => Self::Echo {
                input: val[4..].trim().to_string(),
            },
            val if val.starts_with("type") => Self::Type {
                input: val[4..].trim().to_string(),
            },
            _ => Self::Unknown,
        }
    }
}

#[cfg(unix)]
fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    match path.metadata() {
        Ok(metadata) => metadata.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

fn get_type(input: String) {
    if BUILTINS.contains(&&*input) {
        println!("{} is a shell builtin", input);
        return;
    }

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
            if candidate.is_file() && is_executable(&candidate) {
                println!("{} is {}", input, candidate.display());
                found = true;
                break;
            }
        }

        #[cfg(windows)]
        {
            for ext in &path_exts {
                let candidate = dir.join(format!("{}{}", input, ext));
                if candidate.is_file() {
                    println!("{} is {}", input, candidate.display());
                    found = true;
                    break;
                }
            }
        }
    }

    if !found {
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

        let command_str = input.split(" ").next().unwrap_or("");
        if command_str.is_empty() {
            continue;
        }

        let command = Command::from(input);

        match command {
            Command::Exit => break,
            Command::Echo { input } => println!("{}", input),
            Command::Type { input } => get_type(input),
            Command::Unknown => eprintln!("{}: command not found", command_str.trim()),
        }
    }
}
