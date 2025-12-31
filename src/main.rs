#[allow(unused_imports)]
use std::io::{self, Write};

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
            val if val.starts_with("echo") => {
                Self::Echo { input: val[4..].trim().to_string() }
            },
            val if val.starts_with("type") => {
                Self::Type { input: val[4..].trim().to_string() }
            }
            _ => Self::Unknown
        }
    }
}

fn get_type(input: String) {
    if BUILTINS.contains(&&*input) {
        println!("{} is a shell builtin", input)
    } else {
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
