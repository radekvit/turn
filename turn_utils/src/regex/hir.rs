use std::collections::HashSet;

/// A member of a set. Represents either a single character or a category of characters.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SetMember<'a> {
    Character(char),
    Category(&'a str),
}

/// A high-level representation of a hierarchical regular expression.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HIR<'a> {
    /// Matches any character
    AnyChar,
    /// A sequence of simple characters
    Sequence(&'a str),
    /// A subexpression or character category
    SubRegex(&'a str),
    /// Repetition of a regular expression
    Repetition {
        regex: Box<HIR<'a>>,
        min: u16,
        max: Option<u16>,
    },
    /// Regular expression alternatives
    Alternation(Vec<HIR<'a>>),
    /// A set of characters or character categories
    Set(Vec<SetMember<'a>>),
    /// A set of all characters excluding specific characters or categories
    NegatedSet(Vec<SetMember<'a>>),
    /// Concatenation of regular expressions
    Concatenation(Vec<HIR<'a>>),
}

impl<'a> HIR<'a> {
    pub fn dependencies(&self) -> HashSet<&'a str> {
        use HIR::*;
        match self {
            SubRegex(sub_regex) => {
                let mut dependencies = HashSet::new();
                dependencies.insert(*sub_regex);
                dependencies
            }
            Repetition { regex, .. } => regex.dependencies(),
            Alternation(regexes) | Concatenation(regexes) => {
                regexes.iter().map(HIR::dependencies).flatten().collect()
            }
            Set(variants) | NegatedSet(variants) => variants
                .iter()
                .filter_map(|x| match x {
                    SetMember::Category(c) => Some(*c),
                    _ => None,
                })
                .collect(),
            _ => HashSet::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hir_dependencies() {
        let hir = HIR::Concatenation(vec![
            HIR::AnyChar,
            HIR::SubRegex("dependency1"),
            HIR::Repetition {
                min: 0,
                max: None,
                regex: Box::new(HIR::Alternation(vec![
                    HIR::Set(vec![SetMember::Category("0-9"), SetMember::Character('a')]),
                    HIR::NegatedSet(vec![
                        SetMember::Category("0-9"),
                        SetMember::Category("dependency2"),
                    ]),
                ])),
            },
        ]);

        let mut expected = HashSet::new();
        expected.insert("dependency1");
        expected.insert("dependency2");
        expected.insert("0-9");

        assert_eq!(hir.dependencies(), expected);
    }
}
