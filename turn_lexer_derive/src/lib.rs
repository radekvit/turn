extern crate proc_macro;

//mod matchers;
//mod automata;
mod derive_parse;
//mod lexer_impl;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
//use proc_macro2::TokenStream as TokenStream2;
//use quote::{quote, quote_spanned};
//use syn::{parse_macro_input, DeriveInput, Ident, Data, Fields, Type,PathArguments, GenericArgument,
//    Attribute, Meta, Lit};

#[proc_macro_derive(Lexer, attributes(skip, token, regex))]
pub fn derive(input: TokenStream) -> TokenStream {
    // parse the derive input and process all attributes
    let _input = match derive_parse::parse(parse_macro_input!(input as DeriveInput)) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error().into(),
    };
    // create a minimal finite state automaton from the input
    //let automaton = automata::create_minimal_automaton(input);
    // create turn::Lexer implementation for this enum
    //lexer_impl::create_implementation(automaton)
    TokenStream::new()
}
