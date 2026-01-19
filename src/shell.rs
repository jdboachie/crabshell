mod builtins;
mod input_command;
mod redirection;

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::process::Command;

use anyhow::Error;
use builtins::{CommandType, get_type};
use input_command::InputCommand;
use redirection::{Redirection, check_extract_redirection};

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::path::Path;

#[derive(Default)]
pub struct Shell {
    current_input: String,
    should_quit: bool,
}

impl Shell {
    fn print_or_write(
        std_out_str: Option<String>,
        std_err_str: Option<String>,
        redirection: Option<Redirection>,
    ) {
        match redirection {
            Some(Redirection::StdoutWrite { path }) => {
                let mut f = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)
                    .unwrap();
                if let Some(out) = std_out_str {
                    f.write_all(out.as_bytes()).unwrap();
                }
                if let Some(err) = std_err_str {
                    eprintln!("{}", err.trim_end());
                }
            }

            Some(Redirection::StdoutAppend { path }) => {
                let mut f = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .unwrap();
                if let Some(out) = std_out_str {
                    f.write_all(out.as_bytes()).unwrap();
                }
                if let Some(err) = std_err_str {
                    eprintln!("{}", err.trim_end());
                }
            }

            Some(Redirection::StderrWrite { path }) => {
                let mut f = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)
                    .unwrap();
                if let Some(err) = std_err_str {
                    f.write_all(err.as_bytes()).unwrap();
                }
                if let Some(out) = std_out_str {
                    println!("{}", out.trim_end());
                }
            }

            Some(Redirection::StderrAppend { path }) => {
                let mut f = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .unwrap();
                if let Some(err) = std_err_str {
                    f.write_all(err.as_bytes()).unwrap();
                }
                if let Some(out) = std_out_str {
                    println!("{}", out.trim_end());
                }
            }

            None => {
                if let Some(out) = std_out_str {
                    print!("{}", out);
                }
                if let Some(err) = std_err_str {
                    eprint!("{}", err);
                }
            }
        }
    }

    fn execute(
        &mut self,
        command: InputCommand,
        redirection: Option<Redirection>,
    ) -> Result<(), Error> {
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
            InputCommand::Exit => self.should_quit = true,
            InputCommand::Echo { input } => {
                Self::print_or_write(Some(format!("{}\n", input)), None, redirection);
            }
            InputCommand::Type { input } => {
                let cmd_type = get_type(&input);

                let out_str = match cmd_type {
                    CommandType::Builtin => format!("{} is a shell builtin\n", input),
                    CommandType::Executable { path } => format!("{} is {}\n", input, path),
                    CommandType::Unknown => format!("{}: not found\n", input),
                };

                Self::print_or_write(Some(out_str), None, redirection);
            }
            InputCommand::Pwd => {
                Self::print_or_write(
                    Some(std::env::current_dir().unwrap().display().to_string() + "\n"),
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

                Self::print_or_write(std_out_str, std_err_str, redirection)
            }
            InputCommand::Unknown => {
                eprintln!(
                    "{}: command not found",
                    self.current_input.split(" ").next().unwrap()
                )
            }
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        print!("$ ");
        io::stdout().flush().unwrap();

        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            let mut input_split = shlex::split(input).unwrap_or_default();

            if input_split.is_empty() {
                continue;
            }

            self.current_input = String::from(input);

            let redirection = check_extract_redirection(&mut input_split);
            let command = InputCommand::from(input_split);

            self.execute(command, redirection)?;

            print!("$ ");
            io::stdout().flush().unwrap();
        }
    }
}
