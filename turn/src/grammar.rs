use derive_builder::Builder;

/// A single grammar symbol.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Symbol<Terminal, Nonterminal> {
    Nonterminal(Nonterminal),
    Terminal(Terminal),
}

/// Associativity of a terminal or a group of terminals.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Associativity {
    None,
    Left,
    Right,
}

/// A single context-free grammar rule.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Rule<Terminal, Nonterminal> {
    /// The left-hand nonterminal of the rule.
    left_hand: Nonterminal,
    // The right-hand side of the rule.
    right_hand: Vec<Symbol<Terminal, Nonterminal>>,
    /// The precedence of the rule.
    ///
    /// Usually, this will be set to the last terminal of the right-hand side.
    precedence: Option<Terminal>,
}

impl<Terminal: Copy, Nonterminal> Rule<Terminal, Nonterminal> {
    pub fn new(
        left_hand: Nonterminal,
        right_hand: Vec<Symbol<Terminal, Nonterminal>>,
    ) -> Rule<Terminal, Nonterminal> {
        let precedence = Rule::find_precedence(&right_hand);
        Rule {
            left_hand,
            right_hand,
            precedence,
        }
    }

    pub fn with_precedence(
        left_hand: Nonterminal,
        right_hand: Vec<Symbol<Terminal, Nonterminal>>,
        precedence: Terminal,
    ) -> Rule<Terminal, Nonterminal> {
        Rule {
            left_hand,
            right_hand,
            precedence: Some(precedence),
        }
    }

    fn find_precedence(right_hand: &[Symbol<Terminal, Nonterminal>]) -> Option<Terminal> {
        right_hand.iter().rev().find_map(|e| {
            if let Symbol::Terminal(t) = e {
                Some(*t)
            } else {
                None
            }
        })
    }
}

/// Context-free grammar
#[derive(Clone, Builder, PartialEq, Eq, Debug)]
pub struct Grammar<Terminal, Nonterminal> {
    // The starting nonterminal of the grammar.
    starting_nonterminal: Nonterminal,
    // The rules of this grammar.
    rules: Vec<Rule<Terminal, Nonterminal>>,
    // A list of symbol precedence, starting with the highest precedence.
    symbol_precedence: Vec<(Associativity, Vec<Terminal>)>,
}
