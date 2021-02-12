use std::ffi::OsStr;
use std::fs::{read_to_string, File};
use std::io::Write;

#[test]
fn run_parser_tests() {
    for file in std::fs::read_dir("./tests").unwrap() {
        let path = file.unwrap().path();
        if path.is_file() && path.extension() == Some(OsStr::new("wa")) {
            let content: String = read_to_string(&path).unwrap();
            let content = content.trim_end();

            let lexed = lexer::lex(content);
            assert_eq!(lexed.errors(), vec![]);

            let actual = match super::parse(lexed.tokens()) {
                Ok(items) => format!("{:#?}", items),
                Err(err) => {
                    eprintln!("{}", content);
                    panic!("{}", err);
                }
            };
            let actual = actual.trim_end();

            let ast_path = path.with_extension("ast");
            if ast_path.exists() {
                let expected: String = read_to_string(ast_path).unwrap();
                let expected = expected.trim_end();

                if expected != actual {
                    let changes = difference::Changeset::new(expected, &actual, "\n");
                    eprintln!("{}", changes);
                    eprintln!("Input:\n{}", content);
                    panic!(
                        "{} differences between expected and actual output",
                        changes.distance
                    );
                }
            } else {
                let mut file = File::create(ast_path).unwrap();
                file.write_all(actual.as_bytes()).unwrap();
                file.write_all(b"\n").unwrap();
                file.flush().unwrap();
            }
        }
    }
}
