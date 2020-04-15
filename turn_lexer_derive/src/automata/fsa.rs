use crate::matchers::Matcher;
use std::collections::BTreeMap;

// deterministic finite state automaton
pub struct FSA<Token> {
    current_state: usize,
    states: Vec<FSAState<Token>>,
}

impl<Token: Copy> FSA<Token> {
    pub fn transition(&mut self, c: char) -> bool {
        match self.states[self.current_state].transition(c) {
            Some(x) => {
                self.current_state = x;
                true
            }
            None => false,
        }
    }

    pub fn token(&self) -> Option<Token> {
        self.states[self.current_state].token
    }
}

struct FSAState<Token> {
    transitions: BTreeMap<Matcher, usize>,
    token: Option<Token>,
}

impl<Token: Copy> FSAState<Token> {
    fn transition(&self, c: char) -> Option<usize> {
        for (matcher, &next_state) in &self.transitions {
            if matcher.is_matching(c) {
                return Some(next_state);
            }
        }
        None
    }
}
