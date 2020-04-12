use turn_lexer_derive::Lexer;

#[derive(Lexer)]
#[skip = "{ }"]
enum Foo {
    #[token = "one"]
    One(i32),
    #[regex = "[tT]wo."]
    Two{two: bool},
}

#[derive(Lexer)]
#[skip = "{ }"]
enum Bar {
    #[token = "one"]
    One,
    #[regex = "[tT]wo."]
    Two{two: bool},
}

fn main() {}
