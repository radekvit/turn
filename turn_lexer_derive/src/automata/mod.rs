#[allow(dead_code)]
mod nefsa;

use crate::derive_parse::TerminalEnum;
use nefsa::*;
use syn::{Error, Ident};

pub struct LexerAutomata {
    skip: NEFSA<()>,
    items: Vec<NEFSA<Ident>>,
}

pub struct NFSA;
pub struct FSA;

pub fn create_minimal_automaton(input: TerminalEnum) -> Result<FSA, Error> {
    let _automata = create_automata(&input)?;
    Ok(FSA)
}

fn create_automata(_input: &TerminalEnum) -> Result<LexerAutomata, Error> {
    // create skip regex NEFSA
    // create item NEFSAs
    Ok(LexerAutomata {
        skip: NEFSA { states: vec![] },
        items: vec![],
    })
}
