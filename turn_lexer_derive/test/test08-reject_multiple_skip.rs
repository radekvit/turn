use turn_lexer_derive::Lexer;

#[derive(Lexer)]
#[skip = "420"]
#[skip = "421"]
enum Foo {
    #[token = "one"]
    One,
    #[regex = "[tT]wo."]
    Two,
}

fn main() {}
