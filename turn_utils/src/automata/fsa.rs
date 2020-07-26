use super::hir::*;
use crate::matchers::{Matcher, SingleMatcher};
use fixedbitset::FixedBitSet;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::Display;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct FSA<Token> {
    pub states: Vec<FSAState<Token>>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct FSAState<Token> {
    pub transitions: HashMap<Option<Matcher>, FixedBitSet>,
    pub token: Option<Token>,
}

enum CompileMode {
    Concatenate,
    Separate,
}

impl<Token> FSA<Token>
where
    Token: Clone,
{
    pub fn from_hir(hir: &HIR, token: Token) -> Self {
        let mut result = FSA::from_hir_composite(hir);
        let last = result.states.last_mut().unwrap();
        last.token = Some(token);
        result
    }

    fn from_hir_composite(hir: &HIR) -> Self {
        FSA::compile(FSA::hir_to_fsa_vec(hir), CompileMode::Concatenate)
    }

    fn hir_to_fsa_vec(hir: &HIR) -> Vec<Self> {
        match hir {
            HIR::AnyChar => vec![FSA {
                states: vec![FSAState::with_single_transition(None, 1), FSAState::new()],
            }],
            HIR::Sequence(sequence) => {
                let mut transitions = sequence
                    .chars()
                    .enumerate()
                    .map(|(index, character)| {
                        FSAState::with_single_transition(
                            Some(Matcher::SingleMatcher(SingleMatcher::Character(character))),
                            index + 1,
                        )
                    })
                    .collect::<Vec<_>>();
                transitions.push(FSAState::new());
                vec![FSA {
                    states: transitions,
                }]
            }
            HIR::SubRegex(_) => panic!(
                "Attempting to create an automaton from a regex with unresolved dependencies."
            ),
            HIR::Repetition { regex, min, max } => {
                let mut body = FSA::hir_to_fsa_vec(regex);
                // repeat the body until the minimum has been reached
                let len = body.iter().fold(0, |acc, x| acc + x.states.len());
                // repeat until min has been reached
                if *min != 0 {
                    let cloned = body.clone();
                    for _ in 0..*min {
                        body.extend(cloned.clone());
                    }
                }
                if let Some(max) = max {
                    todo!()
                } else {
                    // allow infinite looping
                    // from the last state to the initial state of the last loop
                    let mut fsa = FSA::compile(body, CompileMode::Concatenate);
                    let last_loop_initial = fsa.states.len() - len;
                    let last_state = fsa.states.len();
                    fsa.states.last_mut().map(|state| {
                        let mut next_states = FixedBitSet::with_capacity(last_state + 1);
                        next_states.insert(last_loop_initial);
                        next_states.insert(last_state);
                        state.transitions.insert(None, next_states);
                    });
                    fsa.states.push(FSAState::new());
                    vec![fsa]
                }
                // compile the intermediate fsa
                // add a transition to the first state of the last repetition if infinite repetitions
                // if finite repetitions, repeat and add escape to each new repetition
            }
            HIR::Alternation(alternatives) => {
                // get each of the variants
                // compile into a single regex, remember the indices of end states
                // calculate the relative position of the new end state for each variant
                // add this new epsilon transition to each variant
                todo!()
            }
            HIR::Set(alternatives) => todo!(),
            HIR::NegatedSet(excluded) => vec![FSA {
                states: vec![
                    FSAState::with_single_transition(
                        Some(Matcher::NegatedSet(
                            excluded
                                .iter()
                                .map(|item| item.try_into().expect("unknown category"))
                                .collect(),
                        )),
                        1,
                    ),
                    FSAState::new(),
                ],
            }],
            HIR::Concatenation(hirs) => hirs.iter().map(FSA::hir_to_fsa_vec).flatten().collect(),
        }
    }

    fn compile(mut fsas: Vec<Self>, mode: CompileMode) -> Self {
        // add offsets to each FSA
        fsas.iter_mut().fold(0, |mut acc, fsa| {
            let states = &mut fsa.states;
            if acc != 0 {
                for state in states.iter() {
                    for (_, next) in &state.transitions {
                        let mut new_next = FixedBitSet::with_capacity(next.len() + acc);
                        next.ones().for_each(|x| new_next.insert(x + acc));
                    }
                }
            }
            match mode {
                CompileMode::Concatenate => {
                    let len = states.len();
                    states.last_mut().map(|last| {
                        let next_state = acc + len;
                        let mut next = FixedBitSet::with_capacity(next_state + 1);
                        next.insert(next_state);
                        last.transitions.insert(None, next);
                    });
                }
                CompileMode::Separate => (),
            }
            acc += states.len();
            acc
        });
        // remove the epsilon transition from the last state
        fsas.last_mut().map(|x| {
            x.states.last_mut().map(|x| x.transitions.remove(&None));
        });
        // flatten the FSAs into a single FSA
        fsas.into_iter()
            .fold(FSA { states: vec![] }, |mut acc, fsa| {
                acc.states.extend(fsa.states.into_iter());
                acc
            })
    }

    pub fn transition(&self, state: usize, c: char) -> FixedBitSet {
        self.states[state].transition(c)
    }

    pub fn token(&self, state: usize) -> Option<&Token> {
        self.states[state].token.as_ref()
    }

    /// Produce a union of multiple automatons by creating a new starting state
    /// with epsilon transitions to all previous starting states.
    ///
    /// Applies offsets to all transitions.
    pub fn union<T>(sources: T) -> Self
    where
        T: IntoIterator<Item = Self>,
    {
        // create first state that has epsilon transitions to all variants
        let mut first_state_epsilon_transitions: FixedBitSet = FixedBitSet::with_capacity(0);
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
                        let set = set.ones().map(|i| i + offset).collect();
                        (matcher.clone(), set)
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
        let mut first_state_transitions = HashMap::new();
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
}

impl<Token> FSAState<Token> {
    fn new() -> Self {
        Self {
            transitions: HashMap::new(),
            token: None,
        }
    }

    fn with_single_transition(matcher: Option<Matcher>, next: usize) -> Self {
        let mut transitions = HashMap::new();
        let mut next_states = FixedBitSet::with_capacity(next + 1);
        next_states.insert(next);
        transitions.insert(matcher, next_states);
        Self {
            transitions,
            token: None,
        }
    }

    fn transition(&self, c: char) -> FixedBitSet {
        let mut result = FixedBitSet::with_capacity(0);
        for (matcher, ref next_states) in &self.transitions {
            if let Some(matcher) = matcher {
                if matcher.is_matching(c) {
                    result.union_with(next_states);
                }
            } else {
                result.union_with(next_states);
            }
        }
        result
    }

    fn epsilon_transitions(&self) -> FixedBitSet {
        let result;
        if let Some(epsilon_transitions) = self.transitions.get(&None) {
            result = epsilon_transitions.clone();
        } else {
            result = FixedBitSet::with_capacity(0);
        }
        result
    }
}

#[derive(Debug)]
pub struct UnknownCategory;

impl Display for UnknownCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown built-in category")
    }
}

impl std::error::Error for UnknownCategory {}

impl<'a> TryFrom<&SetMember<'a>> for SingleMatcher {
    type Error = UnknownCategory;

    fn try_from(value: &SetMember<'a>) -> Result<Self, Self::Error> {
        use crate::matchers::CharacterCategory;

        match value {
            SetMember::Character(char) => Ok(SingleMatcher::Character(*char)),
            SetMember::Category(category) => match *category {
                "lower" => Ok(SingleMatcher::Category(CharacterCategory::Utf8Lowercase)),
                "upper" => Ok(SingleMatcher::Category(CharacterCategory::Utf8Uppercase)),
                "alpha" => Ok(SingleMatcher::Category(CharacterCategory::Utf8Alpha)),
                "alnum" => Ok(SingleMatcher::Category(CharacterCategory::Utf8Alphanumeric)),
                "digit" => Ok(SingleMatcher::Category(CharacterCategory::Utf8Numeric)),
                "whitespace" => Ok(SingleMatcher::Category(CharacterCategory::Utf8Whitespace)),
                "a-z" => Ok(SingleMatcher::Category(CharacterCategory::ASCIILowercase)),
                "A-Z" => Ok(SingleMatcher::Category(CharacterCategory::ASCIIUppercase)),
                "a-Z" => Ok(SingleMatcher::Category(CharacterCategory::ASCIIAlpha)),
                "0-Z" => Ok(SingleMatcher::Category(
                    CharacterCategory::ASCIIAlphanumeric,
                )),
                "0b" => Ok(SingleMatcher::Category(CharacterCategory::ASCIIBinaryDigit)),
                "0-9" => Ok(SingleMatcher::Category(CharacterCategory::ASCIIDigit)),
                "0x" => Ok(SingleMatcher::Category(CharacterCategory::ASCIIHexDigit)),
                " " => Ok(SingleMatcher::Category(CharacterCategory::ASCIIWhitespace)),
                _ => Err(UnknownCategory),
            },
        }
    }
}
