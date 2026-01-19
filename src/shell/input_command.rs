use crate::shell::builtins::find_executable;

pub enum InputCommand {
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
