use turn_lexer_derive::Lexer;

#[derive(Lexer)]
#[skip = "{ }"]
enum Foo {
    #[token = "one"]
    One,
    #[regex = "[tT]wo."]
    Two,
}

fn main() {}
