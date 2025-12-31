#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input: Vec<&str> = input.trim().split(" ").collect();
        let command = input[0];

        if command.is_empty() {
            continue;
        }

        match command {
            "exit" => break,
            cmd if cmd.starts_with("echo") => println!("{}", &cmd[4..].trim()),
            _ => eprintln!("{}: command not found", command.trim()),
        }
    }
}
