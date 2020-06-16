extern crate proc_macro;

use crate::automata::FSA;
use proc_macro::TokenStream;

pub fn create_implementation(_fsa: FSA) -> TokenStream {
    TokenStream::new()
}
