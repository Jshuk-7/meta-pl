use meta::parser::Parser;

fn main() {
    if let Some(mut parser) = Parser::from_file("Script.mt") {
        parser.make_program();
    }
}
