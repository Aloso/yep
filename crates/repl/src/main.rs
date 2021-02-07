use ast::token::TokenKind;
use lexer::Program;
use parser::formatting::ToBeauty;


const BLUE: &str = "\x1b[38;2;50;220;255m";
const GREEN: &str = "\x1b[38;2;80;230;100m";
const YELLOW: &str = "\x1b[38;2;255;235;0m";
const ORANGE: &str = "\x1b[38;2;255;135;0m";
const RED: &str = "\x1b[38;2;255;40;40m";
const PURPLE: &str = "\x1b[38;2;255;70;255m";
const GRAY: &str = "\x1b[38;2;130;130;130m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

fn main() {
    println!("Yep 0.1 REPL. Press Enter twice to validate. Press Ctrl+C to exit.\n");

    let stdin = std::io::stdin();
    loop {
        let mut text = String::new();
        loop {
            stdin.read_line(&mut text).unwrap();
            if text.ends_with("\n\n") {
                let _ = text.pop();
                break;
            }
        }

        print!("Lexed program:  ");
        let program = lexer::lex(&text);
        print_program(&program);
        println!("\n");

        match parser::parse(program.tokens()) {
            Ok(parsed) => {
                println!("Parsed output:");
                println!("{}", parsed.to_beauty_string().trim_end());
                println!("\n");
            }
            Err(error) => {
                if let parser::Error::RemainingTokens(t) = error {
                    let rest = program.with_lifeless_tokens(&t);
                    print!("Expected item, found:  ");
                    print_program(&rest);
                    println!("\n");
                } else {
                    println!("{}\n", error);
                }
            }
        }
    }
}

fn print_program(program: &Program) {
    for k in program.tokens() {
        match k.kind() {
            TokenKind::Punct => print!("{}", GRAY),
            TokenKind::NumberLit => print!("{}", YELLOW),
            TokenKind::StringLit => print!("{}", ORANGE),
            TokenKind::Ident => print!("{}{}", RESET, BOLD),
            TokenKind::UpperIdent => print!("{}", GREEN),
            TokenKind::Operator => print!("{}", PURPLE),
            TokenKind::Keyword => print!("{}", BLUE),
            TokenKind::Error => print!("{}", RED),
            TokenKind::EOF => print!("{}", RED),
        }
        print!("{}{} ", k.debug_to_string(program.text(), false), RESET);
    }
    print!("{}", RESET);
}
