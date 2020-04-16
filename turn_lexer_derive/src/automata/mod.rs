mod dfsa;
mod fsa;

use crate::derive_parse::{Regex, TerminalEnum};
use dfsa::*;
use fsa::*;
//use nfsa::*;
use syn::{Error, Ident};

struct LexerAutomata<'a> {
    skip: FSA<()>,
    items: Vec<FSA<&'a Ident>>,
}

pub fn create_minimal_automaton<'a>(input: &'a TerminalEnum) -> Result<DFSA<&'a Ident>, Error> {
    let automata = create_automata(input)?;
    let _skip = automata.skip;
    let _automaton = FSA::union(automata.items);
    // remove epsilon transitions
    // determinize the automata
    // minimize the automata
    Ok(DFSA { states: vec![] })
}

fn create_automata<'a>(input: &'a TerminalEnum) -> Result<LexerAutomata<'a>, Error> {
    // create skip regex FSA
    let automata: Result<_, _> = input
        .variants
        .iter()
        .map(|(ident, regexes)| {
            regexes.iter().map(move |regex| {
                match regex {
                    Regex::Token(s) => FSA::from_token(ident, s.span, &s.regex),
                    // TODO parse regex
                    Regex::Regex(s) => FSA::from_token(ident, s.span, &s.regex),
                }
            })
        })
        .flatten()
        .collect();
    // create item FSAs
    Ok(LexerAutomata {
        skip: FSA { states: vec![] },
        items: automata?,
    })
}
