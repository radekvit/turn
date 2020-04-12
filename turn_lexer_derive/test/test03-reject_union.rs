use turn_lexer_derive::Lexer;

#[derive(Lexer)]
#[skip = "{ }"]
union Foo {
    #[token = "one"]
    one: bool,
    #[regex = "[tT]wo."]
    two: i32,
}

fn main() {}
