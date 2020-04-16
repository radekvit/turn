#![allow(dead_code)]

extern crate proc_macro;

mod automata;
mod derive_parse;
mod matchers;
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
    let input = match derive_parse::parse(parse_macro_input!(input as DeriveInput)) {
        Ok(input) => input,
        Err(error) => return error.to_compile_error().into(),
    };
    // create a minimal finite state automaton from the input
    let _automaton = match automata::create_minimal_automaton(&input) {
        Ok(automaton) => automaton,
        Err(error) => return error.to_compile_error().into(),
    };
    // create turn::Lexer implementation for this enum
    //lexer_impl::create_implementation(automaton)
    TokenStream::new()
}
