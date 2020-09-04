use super::mir::*;
use crate::matchers::{Matcher, SingleMatcher};
use fixedbitset::FixedBitSet;
use std::collections::HashMap;

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
    pub fn from_mir(mir: &MIR, token: Token) -> Self {
        let mut result = FSA::from_mir_composite(mir);
        let last = result.states.last_mut().unwrap();
        last.token = Some(token);
        result
    }

    fn from_mir_composite(mir: &MIR) -> Self {
        FSA::compile(FSA::mir_to_fsa_vec(mir), CompileMode::Concatenate)
    }

    fn mir_to_fsa_vec(mir: &MIR) -> Vec<Self> {
        match mir {
            MIR::Category(c) => vec![FSA {
                states: vec![
                    FSAState::with_single_transition(
                        Some(Matcher::SingleMatcher(SingleMatcher::Category(*c))),
                        1,
                    ),
                    FSAState::new(),
                ],
            }],
            MIR::Sequence(sequence) => {
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
            MIR::Repetition { regex, min, max } => {
                let mut body = FSA::mir_to_fsa_vec(regex);
                // repeat the body until the minimum has been reached
                let len = body.iter().fold(0, |acc, x| acc + x.states.len());
                let cloned = body.clone();
                let limit = match max {
                    Some(max) => *max,
                    None => *min,
                };
                for _ in 0..limit {
                    body.extend(cloned.clone());
                }
                let mut fsa = FSA::compile(body, CompileMode::Concatenate);
                let last_state = fsa.states.len();
                if let Some(max) = max {
                    // add transitions to last state for all last states between min and max
                    for i in *min..=*max {
                        let state = (i + 1) as usize * len - 1;
                        let mut next_states = FixedBitSet::with_capacity(last_state + 1);
                        next_states.insert(state + 1);
                        next_states.insert(last_state);
                        fsa.states[state].transitions.insert(None, next_states);
                    }
                } else {
                    // add transition to the last state to the first state of the last loop
                    let last_loop_initial = fsa.states.len() - len;
                    let last_state = fsa.states.len();
                    fsa.states.last_mut().map(|state| {
                        let mut next_states = FixedBitSet::with_capacity(last_state + 1);
                        next_states.insert(last_loop_initial);
                        next_states.insert(last_state);
                        state.transitions.insert(None, next_states);
                    });
                }
                // push new last state
                fsa.states.push(FSAState::new());
                vec![fsa]
            }
            MIR::Alternation(alternatives) => {
                // get each of the variants
                // compile into a single regex, remember the indices of end states
                // calculate the relative position of the new end state for each variant
                // add this new epsilon transition to each variant
                let mut subexpressions: Vec<_> =
                    alternatives.iter().map(FSA::from_mir_composite).collect();
                let last_state = subexpressions.iter().fold(1, |acc, x| acc + x.states.len());
                // add transitions to new last state
                subexpressions
                    .iter_mut()
                    .fold(last_state, |mut last_state, x| {
                        last_state -= x.states.len();
                        let mut transition = FixedBitSet::with_capacity(last_state + 1);
                        transition.insert(last_state);
                        x.states
                            .last_mut()
                            .unwrap()
                            .transitions
                            .insert(None, transition);
                        last_state
                    });
                // add transitions from new first state
                let mut transition = FixedBitSet::with_capacity(last_state);
                subexpressions.iter().fold(1, |acc, x| {
                    transition.insert(acc);
                    acc + x.states.len()
                });
                let first_state = FSA {
                    states: vec![FSAState::with_single_matcher(None, transition)],
                };
                subexpressions.insert(0, first_state);
                subexpressions.push(FSA {
                    states: vec![FSAState::new()],
                });
                vec![FSA::compile(subexpressions, CompileMode::Separate)]
            }
            MIR::Set(alternatives) => {
                let next = {
                    let mut next = FixedBitSet::with_capacity(2);
                    next.insert(1);
                    next
                };
                let transitions = {
                    let mut transitions = HashMap::new();
                    alternatives.iter().map(Into::into).for_each(|alternative| {
                        transitions.insert(Some(Matcher::SingleMatcher(alternative)), next.clone());
                    });
                    transitions
                };
                vec![FSA {
                    states: vec![
                        FSAState {
                            transitions,
                            token: None,
                        },
                        FSAState::new(),
                    ],
                }]
            }
            MIR::NegatedSet(excluded) => vec![FSA {
                states: vec![
                    FSAState::with_single_transition(
                        Some(Matcher::NegatedSet(
                            excluded.iter().map(Into::into).collect(),
                        )),
                        1,
                    ),
                    FSAState::new(),
                ],
            }],
            MIR::Concatenation(mirs) => mirs.iter().map(FSA::mir_to_fsa_vec).flatten().collect(),
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

    fn with_single_matcher(matcher: Option<Matcher>, next_states: FixedBitSet) -> Self {
        let mut transitions = HashMap::new();
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
        self.transitions
            .get(&None)
            .map_or_else(|| FixedBitSet::with_capacity(0), Clone::clone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matchers::CharacterCategory;

    #[test]
    fn from_mir_any_char() {
        let mir = MIR::Category(CharacterCategory::Any);
        assert_eq!(
            FSA::from_mir(&mir, ()),
            FSA {
                states: vec![
                    FSAState::with_single_transition(
                        Some(Matcher::SingleMatcher(SingleMatcher::Category(
                            CharacterCategory::Any
                        ))),
                        1
                    ),
                    FSAState {
                        transitions: HashMap::new(),
                        token: Some(())
                    }
                ]
            }
        );
    }

    #[test]
    fn from_mir_sequence() {
        let mir = MIR::Sequence("abc");
        assert_eq!(
            FSA::from_mir(&mir, ()),
            FSA {
                states: vec![
                    FSAState::with_single_transition(
                        Some(Matcher::SingleMatcher(SingleMatcher::Character('a'))),
                        1
                    ),
                    FSAState::with_single_transition(
                        Some(Matcher::SingleMatcher(SingleMatcher::Character('b'))),
                        2
                    ),
                    FSAState::with_single_transition(
                        Some(Matcher::SingleMatcher(SingleMatcher::Character('c'))),
                        3
                    ),
                    FSAState {
                        transitions: HashMap::new(),
                        token: Some(())
                    }
                ]
            }
        );
    }
}
