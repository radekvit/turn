use turn_lexer_derive::Lexer;

#[derive(Lexer)]
#[skip = "420"]
enum Foo {
    #[token = "one"]
    One,
    Two,
    #[regex = "three"]
    Three,
    #[token = "four"]
    #[regex = "(four){4}"]
    Four,
}

fn main() {}
