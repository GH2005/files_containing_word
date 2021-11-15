pub struct RunResult {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

pub fn run(args: std::env::Args) -> RunResult {
    let args: Vec<_> = args.collect();
    if args.len() != 3 {
        return RunResult{ stdout: None, stderr: Some(String::from("Usage: cargo run path_to_dir word_to_search")) };
    }

    let files_and_errors = file_search::files_containing_word(args[1].clone().into(), &args[2]);

    let files = files_and_errors.iter().filter_map(
            |result| match result {
                Ok(path) => Some(path.to_str().unwrap_or("can't display this path")),
                Err(_) => None,
            }
        )
        .fold( String::new(), |accu, ele| accu + ele + "\n" );

    let errors = files_and_errors.iter().filter_map(
            |result| match result {
                Ok(_) => None,
                Err(e) => Some(e.to_string() + "\n"),
            }
        )
        .reduce( |accu, ref ele| accu + ele );

    RunResult{ stdout: Some(files), stderr: errors }
}

mod file_search;