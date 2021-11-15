use std::io::{Error, ErrorKind};
use std::path::PathBuf;

///std::io::Error doesn't contain path info, hence this type.
#[derive(Debug)]
pub struct ErrorWithPath {
    pub error: Error,
    pub path: PathBuf,
}

impl std::fmt::Display for ErrorWithPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path.as_path().display(), self.error)
    }
}

///every access to the file system can potentially fail, therefore the following code matches against Ok and Err on each step.
pub type Result = std::result::Result<PathBuf, ErrorWithPath>; //a sum type

///dir has to identify a directory; a valid regular file won't do.
pub fn files_containing_word(dir: PathBuf, word: &str) -> Vec<Result> {
    let path_result =
        if dir.is_dir() { Ok(dir) }
        else { Err(ErrorWithPath{ error: Error::new(ErrorKind::InvalidInput, NON_DIR_ERROR_MSG), path: dir }) };

    files_at_path(path_result).into_iter().filter_map(
            |result| match result {
                Ok(file) => {
                    match std::fs::read_to_string(&file) {
                        Ok(contents) => contents.find(word).and(Some(Ok(file))),
                        Err(e) => Some(Err(ErrorWithPath{ error: e, path: file })),
                    }
                }
                Err(error_with_path) => {
                    Some(Err(error_with_path))
                },
            }
        )
        .collect()
}

const NON_DIR_ERROR_MSG: &str = "path doesn't identify an accessible directory";

///serves as the base case of the recursion.
fn wrap_into_vec<T>(t: T) -> Vec<T> {
    vec![t]
}

///input_result can identify a directory, a regular file, or an error.
fn files_at_path(input_result: Result) -> Vec<Result> {
    match input_result {
        Ok(path) => {//access the path
            match path.metadata() {
                Ok(meta) => {

                    if meta.is_dir() {
                        match std::fs::read_dir(&path) {
                            Ok(read_dir) => {
                                read_dir.map(
                                        |entry_result| {
                                            let converted_result = match entry_result {
                                                    Ok(entry) => Ok(entry.path()),
                                                    Err(e) => Err(ErrorWithPath{ error: e, path: String::from("Unknown path").into() }),
                                                        // ^^^ I can find no way to obtain a valid path here, hence the "Unknown path" literal.
                                                };
                                            files_at_path(converted_result)
                                        }
                                    )
                                    .flatten()
                                    .collect()
                            },
                            Err(e) => {
                                wrap_into_vec(Err(ErrorWithPath{ error: e, path }))
                            },
                        }
                    }
                    else if meta.is_file() {
                        wrap_into_vec(Ok(path))
                    }
                    else {
                        unreachable!()
                    }

                },
                Err(e) => {
                    wrap_into_vec(Err(ErrorWithPath{ error: e, path }))
                },
            }
        },
        Err(error_with_path) => {//or keep the error as is
            wrap_into_vec(Err(error_with_path))
        },
    }
}

#[cfg(test)]
mod test_mod {
    use super::*;
    const TEST_DATA_PATH: &str = "test_data/";
    const ERROR_TEXT: &str = "error text";
    const INVALID_PATH: &str = "nowhere";
    const EMPTY_WORD: &str = "";
    const WORD: &str = "wo rd";

    #[test]
    fn file_list_test() {
        //an error as input
        let error: ErrorWithPath = ErrorWithPath{ error: Error::new(ErrorKind::InvalidInput, ERROR_TEXT), path: TEST_DATA_PATH.into() };
        let files = files_at_path(Err(error));
        assert_eq!(files.len(), 1);
        assert!(matches!(&files[0], Err(ErrorWithPath{ error: e, path: p }) if e.kind() == ErrorKind::InvalidInput && e.to_string() == ERROR_TEXT && p.to_str().unwrap() == TEST_DATA_PATH));

        //an invalid path as input
        let files = files_at_path(Ok(INVALID_PATH.into()));
        assert_eq!(files.len(), 1);
        assert!(matches!(&files[0], Err(ErrorWithPath{ error: e, path: p }) if e.kind() == ErrorKind::NotFound && p.to_str().unwrap() == INVALID_PATH));

        //a text file as input
        let file_path: PathBuf = (TEST_DATA_PATH.to_owned() + "file0_0.txt").into();
        let files = files_at_path(Ok(file_path.clone()));
        assert_eq!(files.len(), 1);
        assert!(matches!(&files[0], Ok(p) if p == &file_path));

        //a binary file as input
        let file_path: PathBuf = (TEST_DATA_PATH.to_owned() + "binary_file0_0").into();
        let files = files_at_path(Ok(file_path.clone()));
        assert_eq!(files.len(), 1);
        assert!(matches!(&files[0], Ok(p) if p == &file_path));

        //an inaccessible dir as input
        let dir_path: PathBuf = (TEST_DATA_PATH.to_owned() + "dir0_0_inaccessible").into();
        let files = files_at_path(Ok(dir_path.clone()));
        assert_eq!(files.len(), 1);
        assert!(matches!(&files[0], Err(ErrorWithPath{ error: e, path: p }) if e.kind() == ErrorKind::PermissionDenied && p.to_str().unwrap() == dir_path.to_str().unwrap()));

        //an empty dir as input
        let dir_path: PathBuf = (TEST_DATA_PATH.to_owned() + "dir0_1_empty").into();
        let files = files_at_path(Ok(dir_path.clone()));
        assert_eq!(files.len(), 0);

        //a dir containing an inaccessible file as input
        let dir_path: PathBuf = (TEST_DATA_PATH.to_owned() + "dir0_2").into();
        let files = files_at_path(Ok(dir_path.clone()));
        assert_eq!(files.len(), 1);
        assert!(matches!(&files[0], Ok(p) if p.to_str().unwrap() == dir_path.to_str().unwrap().to_owned() + "/file1_0_inaccessible.txt"));

        //a dir containing two regular files and a dir with one file as input
        let dir_path: PathBuf = (TEST_DATA_PATH.to_owned() + "dir0_3").into();
        let files = files_at_path(Ok(dir_path.clone()));
        assert_eq!(files.len(), 3);
        let file_count = files.iter().filter( |&r| r.is_ok() ).count();
        let error_count = files.iter().filter( |&r| r.is_err() ).count();
        assert_eq!(file_count, 3);
        assert_eq!(error_count, 0);

        //same as above, but via a symlink
        let dir_path: PathBuf = (TEST_DATA_PATH.to_owned() + "dir0_3_symlink").into();
        let files = files_at_path(Ok(dir_path.clone()));
        assert_eq!(files.len(), 3);
        let file_count = files.iter().filter( |&r| r.is_ok() ).count();
        let error_count = files.iter().filter( |&r| r.is_err() ).count();
        assert_eq!(file_count, 3);
        assert_eq!(error_count, 0);

        //the whole test folder
        let dir_path: PathBuf = TEST_DATA_PATH.into();
        let files = files_at_path(Ok(dir_path.clone()));
        assert_eq!(files.len(), 10);
        let file_count = files.iter().filter( |&r| r.is_ok() ).count();
        let error_count = files.iter().filter( |&r| r.is_err() ).count();
        assert_eq!(file_count, 9);
        assert_eq!(error_count, 1);
    }

    #[test]
    fn word_filter_test() {
        //non-dir input
        let file_path: PathBuf = (TEST_DATA_PATH.to_owned() + "file0_0.txt").into();
        let files = files_containing_word(file_path.clone(), EMPTY_WORD);
        assert_eq!(files.len(), 1);
        assert!(matches!(&files[0], Err(ErrorWithPath{ error: e, path: p }) if e.kind() == ErrorKind::InvalidInput && e.to_string() == NON_DIR_ERROR_MSG && p.to_str().unwrap() == file_path.to_str().unwrap()));

        //empty word
        let dir_path: PathBuf = TEST_DATA_PATH.into();
        let files = files_containing_word(dir_path.clone(), EMPTY_WORD);
        assert_eq!(files.len(), 10);
        let file_count = files.iter().filter( |&r| r.is_ok() ).count();
        let error_count = files.iter().filter( |&r| r.is_err() ).count();
        assert_eq!(file_count, 7);
        assert_eq!(error_count, 3);

        //non-empty word
        let dir_path: PathBuf = TEST_DATA_PATH.into();
        let files = files_containing_word(dir_path.clone(), WORD);
        assert_eq!(files.len(), 6);
        let file_count = files.iter().filter( |&r| r.is_ok() ).count();
        let error_count = files.iter().filter( |&r| r.is_err() ).count();
        assert_eq!(file_count, 3);
        assert_eq!(error_count, 3);
    }
}
