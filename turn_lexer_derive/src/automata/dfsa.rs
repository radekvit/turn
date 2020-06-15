use crate::automata::fsa::FSA;
use crate::matchers::Matcher;
use std::collections::{BTreeMap, BTreeSet};

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

    pub fn remove_unreachable_states(&mut self)
    where
        Token: Copy,
    {
        let mut state_ids = BTreeMap::new();
        let mut new_states = BTreeSet::new();
        new_states.insert(0usize);
        let mut next_state = 0;
        // rename all reachable states
        while !new_states.is_empty() {
            let mut states = BTreeSet::new();
            for state in new_states.iter() {
                // insert this state
                state_ids.insert(*state, next_state);
                next_state += 1;
                // get all following states
                let state = &self.states[*state];
                state.transitions.iter().for_each(|(_, next_state)| {
                    states.insert(*next_state);
                });
            }
            std::mem::swap(&mut new_states, &mut states);
        }
        // remove unreachable states and rename transitions
        self.states = self
            .states
            .iter()
            .enumerate()
            .filter_map(|(i, state)| {
                if state_ids.get(&i).is_some() {
                    let mut transitions = BTreeMap::new();
                    state.transitions.iter().for_each(|(matcher, next)| {
                        transitions.insert(*matcher, *state_ids.get(next).unwrap());
                    });
                    Some(DFSAState {
                        transitions,
                        token: state.token,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

impl<Token: Clone> From<&FSA<Token>> for DFSA<Token> {
    fn from(_other: &FSA<Token>) -> DFSA<Token> {
        DFSA { states: vec![] }
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
