//mod finite_automata;
pub mod grammar;
pub mod lexer;
//pub mod parser;

pub use lexer::*;

pub fn parse<'a, 'b, Symbol, Lexer>(_lexer: Lexer)
where
    Lexer: Iterator<Item = Result<lexer::Token<'a, 'b, Symbol>, ()>>,
{
    unimplemented!();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
