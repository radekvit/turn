// nondeterministic finite state automaton
use std::collections::BTreeSet;

use super::matchers::Matcher;

pub struct Transitions {
    transitions: Vec<(Matcher, BTreeSet<usize>)>,
}

pub struct NFSA<Token: Ord> {
    states: Vec<(Transitions, Option<Token>)>,
}

impl<Token: Ord> NFSA<Token> {
    pub fn new(states: Vec<(Transitions, Option<Token>)>) -> Self {
        NFSA { states }
    }

    pub fn transitions(&self, state: usize, c: char) -> Option<&BTreeSet<usize>> {
        let transitions = &self.states[state].0.transitions;
        for (matcher, states) in transitions.iter() {
            if matcher.is_matching(c) {
                return Some(states);
            }
        }
        None
    }
}
