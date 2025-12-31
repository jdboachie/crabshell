#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let command = command.trim();

        if command.is_empty() {
            continue;
        }

        match command {
            "exit" => break,
            _ => eprintln!("{}: command not found", command.trim())
        }
    }
}
