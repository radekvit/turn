use crate::matchers::Matcher;
use std::collections::{BTreeMap, BTreeSet};

// Nondeterministic FSA without epsilon transitions
pub struct NFSA<Token> {
    states: Vec<NFSAState<Token>>,
}

struct NFSAState<Token> {
    transitions: BTreeMap<Matcher, BTreeSet<usize>>,
    token: Option<Token>,
}
