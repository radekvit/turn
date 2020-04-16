use crate::matchers::Matcher;
use proc_macro2::Span;
use std::collections::{BTreeMap, BTreeSet};
use std::iter::IntoIterator;
use syn::Error;

pub struct FSA<Token> {
    pub states: Vec<FSAState<Token>>,
}

impl<Token> FSA<Token> {
    /// Parse a token specification and create a nondeterministic finite state automaton
    /// with epsilon transitions from it.
    ///
    /// Tokens match the literal characters, so the source is directly translated into an automaton.
    pub fn from_token(result: Token, span: Span, source: &str) -> Result<FSA<Token>, Error> {
        if !source.is_empty() {
            let mut states: Vec<_> = source
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    let mut state = FSAState {
                        transitions: BTreeMap::new(),
                        token: None,
                    };
                    let next = {
                        let mut next = BTreeSet::new();
                        next.insert(i + 1);
                        next
                    };
                    state.transitions.insert(Some(Matcher::Character(c)), next);
                    state
                })
                .collect();
            states.push(FSAState {
                transitions: BTreeMap::new(),
                token: Some(result),
            });
            Ok(FSA { states })
        } else {
            Err(Error::new(span, "Token source string must not be empty."))
        }
    }

    /// Parse a regex specification and create a nondeterministic finite state automaton
    /// with epsilon transitions from it.
    ///
    /// We parse the regex format specified in README.md and create the automaton.
    pub fn from_regex(_result: Token, _span: Span, _source: &str) -> Result<FSA<Token>, Error> {
        unimplemented!();
    }

    /// Produce a union of multiple automatons by creating a new starting state
    /// with epsilon transitions to all previous starting states.
    ///
    /// Applies offsets to all transitions.
    pub fn union<T>(sources: T) -> FSA<Token>
    where
        T: IntoIterator<Item = FSA<Token>>,
    {
        // create first state that has epsilon transitions to all variants
        let mut first_state_epsilon_transitions: BTreeSet<usize> = BTreeSet::new();
        let mut states = vec![];
        // append all other states, increasing all values in subsequent merged automatons
        let mut offset = 1;
        for source in sources {
            first_state_epsilon_transitions.insert(offset);
            let state_count = source.states.len();
            // apply offset to all transitions in the source
            let with_offset = source.states.into_iter().map(|state| {
                let transitions = state
                    .transitions
                    .iter()
                    .map(|(matcher, set)| {
                        let set = set.iter().map(|i| *i + offset).collect();
                        (*matcher, set)
                    })
                    .collect();
                FSAState {
                    transitions,
                    token: state.token,
                }
            });
            states.extend(with_offset);
            offset += state_count;
        }
        // create first state transitions
        let mut first_state_transitions = BTreeMap::new();
        first_state_transitions.insert(None, first_state_epsilon_transitions);
        // create new starting state state
        let first_state = FSAState {
            transitions: first_state_transitions,
            token: None,
        };
        // insert new starting state
        states.insert(0, first_state);
        FSA { states }
    }

    pub fn transition(&self, state: usize, c: char) -> BTreeSet<usize> {
        self.states[state].transition(c)
    }

    pub fn token(&self, state: usize) -> Option<&Token> {
        self.states[state].token.as_ref()
    }
}

#[derive(Clone)]
pub struct FSAState<Token> {
    pub transitions: BTreeMap<Option<Matcher>, BTreeSet<usize>>,
    pub token: Option<Token>,
}

impl<Token> FSAState<Token> {
    fn transition(&self, c: char) -> BTreeSet<usize> {
        let result = BTreeSet::new();
        for (matcher, ref next_states) in &self.transitions {
            if let Some(matcher) = matcher {
                if matcher.is_matching(c) {
                    result.union(next_states);
                }
            } else {
                result.union(next_states);
            }
        }
        result
    }

    fn epsilon_transitions(&self) -> BTreeSet<usize> {
        let result;
        if let Some(epsilon_transitions) = self.transitions.get(&None) {
            result = epsilon_transitions.clone();
        } else {
            result = BTreeSet::new();
        }
        result
    }
}

fn regex_escape_sequence(c: char, span: Span) -> Result<char, Error> {
    match c {
        '(' | ')' | '{' | '}' | '<' | '>' | '*' | '+' | '?' | '|' | '.' => Ok(c),
        _ => Err(Error::new(span, format!("Invalid escaped character {}", c))),
    }
}
