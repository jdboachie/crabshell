// const REDIRECTION_FLAGS: [&str; 6] = [">", ">>", "1>", "1>>", "2>", "2>>"];

pub enum Redirection {
    StdoutWrite { path: String },
    StderrWrite { path: String },
    StdoutAppend { path: String },
    StderrAppend { path: String },
}

pub fn check_extract_redirection(input_split: &mut Vec<String>) -> Option<Redirection> {
    let maybe_pos = input_split
        .clone()
        .into_iter()
        .position(|s| s == ">" || s == ">>" || s == "1>" || s == "1>>" || s == "2>" || s == "2>>");

    if let Some(pos) = maybe_pos {
        let res: Option<Redirection>;
        let out_path = Some(input_split[pos + 1].clone()).unwrap();

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
