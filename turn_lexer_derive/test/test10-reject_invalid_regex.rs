use turn_lexer_derive::Lexer;

#[derive(Lexer)]
#[skip = "420"]
enum Foo {
    #[token = 420]
    One,
}

#[derive(Lexer)]
#[skip = "420"]
enum Bar {
    #[token = "420"]
    One,
    #[token]
    Two,
}

#[derive(Lexer)]
#[skip = "420"]
enum Baz {
    #[regex ("three", "four")]
    Three,
}

#[derive(Lexer)]
#[skip = "420"]
enum Quux {
    #[regex]
    Three,
}

#[derive(Lexer)]
#[skip = "420"]
enum Quuxplosion {
    #[skip = "abcd"]
    Three,
}

fn main() {}
