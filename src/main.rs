fn main() {
    let run_result = files_containing_word::run(std::env::args());
    if let Some(out) = &run_result.stdout {
        println!("Files containing the word:\n{}", out);
    }
    if let Some(err) = &run_result.stderr {
        eprintln!("Errors:\n{}", err);
    }
}