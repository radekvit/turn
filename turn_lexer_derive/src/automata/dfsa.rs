use crate::matchers::Matcher;
use std::collections::BTreeMap;

// deterministic finite state automaton
pub struct DFSA<Token> {
    pub states: Vec<DFSAState<Token>>,
}

impl<Token: Copy> DFSA<Token> {
    pub fn transition(&self, state: usize, c: char) -> Option<usize> {
        self.states[state].transition(c)
    }

    pub fn token(&self, state: usize) -> Option<Token> {
        self.states[state].token
    }
}

pub struct DFSAState<Token> {
    pub transitions: BTreeMap<Matcher, usize>,
    pub token: Option<Token>,
}

impl<Token: Copy> DFSAState<Token> {
    fn transition(&self, c: char) -> Option<usize> {
        for (matcher, &next_state) in &self.transitions {
            if matcher.is_matching(c) {
                return Some(next_state);
            }
        }
        None
    }
}
