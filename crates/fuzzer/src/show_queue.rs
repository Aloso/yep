use std::fs::{read_dir, DirEntry, File};
use std::io;
use std::io::Read;
use std::path::PathBuf;

use lexer::Program;

fn main() {
    let path = std::env::args().nth(1);
    match main_(path.as_deref().unwrap_or("out/queue")) {
        Ok(()) => {}
        Err(e) => eprintln!("{}", e),
    }
}

fn main_(dir: &str) -> io::Result<()> {
    let mut files = read_dir(dir)?.filter_map(is_file).collect::<io::Result<Vec<_>>>()?;
    files.sort_unstable();

    for path in files {
        println!("{:?}", path);
        let program = get_program(path)?;

        println!("{}\n", program);
    }
    Ok(())
}

fn is_file(entry: io::Result<DirEntry>) -> Option<io::Result<PathBuf>> {
    match entry {
        Ok(e) => {
            let path = e.path();
            if path.is_file() {
                Some(Ok(path))
            } else {
                None
            }
        }
        Err(e) => Some(Err(e)),
    }
}

fn get_program(path: PathBuf) -> io::Result<Program> {
    let mut buf = Vec::new();
    File::open(path)?.read_to_end(&mut buf)?;

    Ok(match fuzzer::get_tokens(&buf) {
        Ok(program) => program,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
    })
}
