use turn_lexer_derive::Lexer;

#[derive(Lexer)]
#[skip = "{ }"]
#[token = "one"]
enum Foo {
    #[token = "one"]
    One,
    #[regex = "[tT]wo."]
    Two,
}

#[derive(Lexer)]
#[skip = "{ }"]
#[regex = "[tT]wo."]
enum Bar {
    #[token = "one"]
    One,
    #[regex = "[tT]wo."]
    Two,
}

fn main() {}
