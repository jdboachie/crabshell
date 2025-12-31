#[allow(unused_imports)]
use std::io::{self, Write};

fn echo(input: Vec<&str>) {
    for (idx, arg) in input[1..].iter().enumerate() {
        if idx > 0 {
            print!(" ")
        }
        print!("{}", arg);
    }
    println!();
}

fn get_type(input: Vec<&str>) {
    let command = input.get(1);
    match command {
        Some(command) => {
            match *command {
                "echo" | "exit" | "type" => println!("{}: is a shell builtin", command) ,
                _ => println!("{}: not found", command)
            }
        },
        None => println!("No command provided"),
    }
}

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
            "echo" => echo(input),
            "type" => get_type(input),
            _ => eprintln!("{}: command not found", command.trim()),
        }
    }
}
