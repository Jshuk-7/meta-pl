use meta::lexer::Lexer;

fn main() {
    if let Ok(source) = std::fs::read_to_string("Script.mt") {
        let mut lexer = Lexer::new(source);
        lexer.make_tokens();

        println!("{:#?}", lexer.get_tokens().len());
    }
}
