mod finite_automata;
mod lexer;
pub mod matchers;

pub use lexer::*;

use proc_macro::TokenStream;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
