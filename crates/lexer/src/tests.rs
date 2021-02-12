use std::ffi::OsStr;
use std::fs::{read_to_string, File};
use std::io::Write;

#[test]
fn run_lexer_tests() {
    for file in std::fs::read_dir("./tests").unwrap() {
        let path = file.unwrap().path();
        if path.is_file() && path.extension() == Some(OsStr::new("wa")) {
            let content: String = read_to_string(&path).unwrap();
            let content = content.trim_end();

            let p = crate::lex(&content);
            let formatted = format!("{:<#?}", p);

            let tokens_path = path.with_extension("tokens");
            if tokens_path.exists() {
                let expected: String = read_to_string(tokens_path).unwrap();
                let expected = expected.trim_end();

                if expected != formatted {
                    let changes = difference::Changeset::new(expected, &formatted, "\n");
                    eprintln!("{}", changes);
                    eprintln!("Input: {:?}\n{}", path, content);
                    panic!(
                        "{} differences between expected and actual output",
                        changes.distance
                    );
                }
            } else {
                let mut file = File::create(tokens_path).unwrap();
                file.write_all(formatted.as_bytes()).unwrap();
                file.write_all(b"\n").unwrap();
                file.flush().unwrap();
            }
        }
    }
}
