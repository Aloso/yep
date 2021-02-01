use afl::{__fuzz, fuzz};
use string_interner::StringInterner;
use wamos_parser::{lexer, parser};

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(text) = std::str::from_utf8(data) {
            let mut interner = StringInterner::new();
            let program = lexer::lex_with_interner(&text, &mut interner);
            let _ = parser::parse(program.tokens());
        }
    });
}
