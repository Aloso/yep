use afl::{__fuzz, fuzz};
use ast::StringInterner;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(text) = std::str::from_utf8(data) {
            let mut interner = StringInterner::new();
            let program = lexer::lex_with_interner(&text, &mut interner);
            let _ = parser::parse(program.tokens());
        }
    });
}
