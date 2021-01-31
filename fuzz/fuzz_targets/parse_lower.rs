#![no_main]

use libfuzzer_sys::fuzz_target;
use wamos_parser::{lexer, parser, StringInterner};

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let mut interner = StringInterner::new();
        let program = lexer::lex_with_interner(&text, &mut interner);
        let _ = parser::parse(program.tokens());
    }
});
