use turn_lexer_derive::Lexer;

#[derive(Lexer)]
#[skip = 420]
enum Foo {
    #[token = "one"]
    One,
    #[regex = "[tT]wo."]
    Two,
}

#[derive(Lexer)]
#[skip ("420", "b")]
enum Bar {
    #[token = "one"]
    One,
    #[regex = "[tT]wo."]
    Two,
}

#[derive(Lexer)]
#[skip]
enum Baz {
    #[token = "one"]
    One,
    #[regex = "[tT]wo."]
    Two,
}

fn main() {}
