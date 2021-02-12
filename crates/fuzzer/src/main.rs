fn main() {
    afl::fuzz(true, |data| {
        if let Ok(program) = fuzzer::get_tokens(data) {
            if program.errors().is_empty() {
                if let Ok(_items) = parser::parse(program.tokens()) {
                    fuzzer::is_balanced(program.tokens()).unwrap();
                    // if !items.is_empty() {
                    //     panic!();
                    // }
                }
            }
        }
    });
}
