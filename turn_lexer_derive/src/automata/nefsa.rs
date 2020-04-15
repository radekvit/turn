use crate::matchers::Matcher;
use proc_macro2::Span;
use std::collections::{BTreeMap, BTreeSet};
use syn::Error;

pub struct NEFSA<Token> {
    pub states: Vec<NEFSAState<Token>>,
}

impl<Token: Copy> NEFSA<Token> {
    /// Parse a token specification and create a nondeterministic finite state automaton
    /// with epsilon transitions from it.
    ///
    /// Tokens match the literal characters, so the source is directly translated into an automaton.
    pub fn from_token(result: Token, span: Span, source: &str) -> Result<NEFSA<Token>, Error> {
        if !source.is_empty() {
            let mut states: Vec<_> = source
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    let mut state = NEFSAState {
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
            states.push(NEFSAState {
                transitions: BTreeMap::new(),
                token: Some(result),
            });
            Ok(NEFSA { states })
        } else {
            Err(Error::new(span, "Token source string must not be empty."))
        }
    }

    /// Parse a regex specification and create a nondeterministic finite state automaton
    /// with epsilon transitions from it.
    ///
    /// We parse the regex format specified in README.md and create the automaton.
    pub fn from_regex(_result: Token, _span: Span, _source: &str) -> Result<NEFSA<Token>, Error> {
        unimplemented!();
    }

    /// Produce a union of multiple automatons by creating a new starting state
    /// with epsilon transitions to all previous starting states.
    ///
    /// Applies offsets to all transitions.
    pub fn union(sources: &mut [NEFSA<Token>]) -> NEFSA<Token> {
        // create first state that has epsilon transitions to all variants
        let first_state_epsilon_transitions: BTreeSet<usize> = sources
            .iter()
            .scan(0, |sum, source| {
                let result = *sum;
                *sum += source.states.len();
                Some(result)
            })
            .map(|x| x + 1)
            .collect();
        let first_state_transitions = {
            let mut first_state_transitions = BTreeMap::new();
            first_state_transitions.insert(None, first_state_epsilon_transitions);
            first_state_transitions
        };
        let first_state = NEFSAState {
            transitions: first_state_transitions,
            token: None,
        };
        let mut nefsa = NEFSA {
            states: vec![first_state],
        };
        // append all other states, increasing all values in subsequent merged automatons
        let mut offset = 1;
        for source in sources.iter_mut() {
            // apply offset to all transitions in the source
            let with_offset = source.states.iter().map(|state| {
                let transitions = state
                    .transitions
                    .iter()
                    .map(|(matcher, set)| {
                        let set = set.iter().map(|i| *i + offset).collect();
                        (*matcher, set)
                    })
                    .collect();
                NEFSAState {
                    transitions,
                    token: state.token,
                }
            });
            nefsa.states.extend(with_offset);
            offset += source.states.len();
        }
        nefsa
    }

    pub fn transition(&self, state: usize, c: char) -> BTreeSet<usize> {
        self.states[state].transition(c)
    }

    pub fn token(&self, state: usize) -> Option<Token> {
        self.states[state].token
    }
}

/*

let mut states = vec![NEFSAState {
    transitions: BTreeMap::new(),
    token: None,
}];
let mut iter = source.chars();
let mut escape = false;
while let Some(c) = iter.next() {
    if c == '\\' && !escape {
        escape = true;
        continue;
    }
    let matcher = if escape {
        escape = false;
        Matcher::Character(escape_sequence(c, span)?)
    } else {
        Matcher::Character(c)
    };
    let mut next = BTreeSet::new();
    next.insert(states.len());
    states
        .last_mut()
        .unwrap()
        .transitions
        .insert(Some(matcher), next);
    states.push(NEFSAState {
        transitions: BTreeMap::new(),
        token: None,
    });
}
states.last_mut().unwrap().token = Some(result);
*/

#[derive(Clone)]
pub struct NEFSAState<Token> {
    pub transitions: BTreeMap<Option<Matcher>, BTreeSet<usize>>,
    pub token: Option<Token>,
}

impl<Token: Copy> NEFSAState<Token> {
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
}

fn regex_escape_sequence(c: char, span: Span) -> Result<char, Error> {
    match c {
        '(' | ')' | '{' | '}' | '<' | '>' | '*' | '+' | '?' | '|' | '.' => Ok(c),
        _ => Err(Error::new(span, format!("Invalid escaped character {}", c))),
    }
}
