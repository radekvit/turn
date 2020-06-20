pub mod hir;
mod lexer;
//mod parser;

pub use hir::HIR;
use lexer::CategoryLexer;
use lexer::Lexer;
/*pub use parser::Error;

/// Parse a regular expression and return its high-level intermediate representation.
pub fn parse_regex(regex: &str) -> Result<HIR, Error> {
    parser::parse_regex(Lexer::new(regex))
}

/// TODO add documentation
pub fn parse_category(regex: &str) -> Result<HIR, Error> {
    parser::parse_category(CategoryLexer::new(regex))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::SetMember;
    #[test]
    fn parse_sequence() {
        let hir = parse_regex("abcd").expect("Failed to parse");
        assert_eq!(hir, HIR::Sequence("abcd"));
    }

    #[test]
    fn parse_any_char() {
        let hir = parse_regex("_").expect("Failed to parse");
        assert_eq!(hir, HIR::AnyChar);
    }

    #[test]
    fn parse_star_repetition() {
        let hir = parse_regex("a*").expect("Failed to parse");
        assert_eq!(
            hir,
            HIR::Repetition {
                regex: Box::new(HIR::Sequence("a")),
                min: 0,
                max: None,
            }
        );
    }

    #[test]
    fn parse_plus_repetition() {
        let hir = parse_regex("a+").expect("Failed to parse");
        assert_eq!(
            hir,
            HIR::Repetition {
                regex: Box::new(HIR::Sequence("a")),
                min: 1,
                max: None,
            }
        );
    }

    #[test]
    fn parse_question_mark_repetition() {
        let hir = parse_regex("a?").expect("Failed to parse");
        assert_eq!(
            hir,
            HIR::Repetition {
                regex: Box::new(HIR::Sequence("a")),
                min: 0,
                max: Some(1),
            }
        );
    }

    #[test]
    fn parse_m_repetitions() {
        let hir = parse_regex("a{42}").expect("Failed to parse");
        assert_eq!(
            hir,
            HIR::Repetition {
                regex: Box::new(HIR::Sequence("a")),
                min: 42,
                max: Some(42),
            }
        );
    }

    #[test]
    fn parse_m_to_n_repetitions() {
        let hir = parse_regex("a{6-9}").expect("Failed to parse");
        assert_eq!(
            hir,
            HIR::Repetition {
                regex: Box::new(HIR::Sequence("a")),
                min: 6,
                max: Some(9),
            }
        );
    }

    #[test]
    fn parse_zero_to_n_repetitions() {
        let hir = parse_regex("a{-9}").expect("Failed to parse");
        assert_eq!(
            hir,
            HIR::Repetition {
                regex: Box::new(HIR::Sequence("a")),
                min: 0,
                max: Some(9),
            }
        );
    }

    #[test]
    fn parse_m_to_inf_repetitions() {
        let hir = parse_regex("a{6-}").expect("Failed to parse");
        assert_eq!(
            hir,
            HIR::Repetition {
                regex: Box::new(HIR::Sequence("a")),
                min: 6,
                max: None,
            }
        );
    }

    #[test]
    fn parse_escaped_special_characters() {
        let hir = parse_regex("\\_\\*\\+\\{\\}\\\\\\[\\]\\|\\(\\)\\<\\>").expect("Failed to parse");
        assert_eq!(
            hir,
            HIR::Concatenation(vec![
                HIR::Sequence("_"),
                HIR::Sequence("*"),
                HIR::Sequence("+"),
                HIR::Sequence("{"),
                HIR::Sequence("}"),
                HIR::Sequence("\\"),
                HIR::Sequence("["),
                HIR::Sequence("]"),
                HIR::Sequence("|"),
                HIR::Sequence("("),
                HIR::Sequence(")"),
                HIR::Sequence("<"),
                HIR::Sequence(">"),
            ])
        );
    }

    #[test]
    fn parse_set() {
        let hir = parse_regex("[a<cat>.[\\]]").expect("Failed to parse");
        assert_eq!(
            hir,
            HIR::Set(vec![
                SetMember::Character('a'),
                SetMember::Category("cat"),
                SetMember::Character('.'),
                SetMember::Character('['),
                SetMember::Character(']'),
            ])
        );
    }

    #[test]
    fn parse_negative_set() {
        let hir = parse_regex("[!a<cat>.[\\]]").expect("Failed to parse");
        assert_eq!(
            hir,
            HIR::NegatedSet(vec![
                SetMember::Character('a'),
                SetMember::Category("cat"),
                SetMember::Character('.'),
                SetMember::Character('['),
                SetMember::Character(']'),
            ])
        );
    }

    #[test]
    fn parse_alternatives() {
        let hir = parse_regex("a|(bc)*|def").expect("Failed to parse");
        assert_eq!(
            hir,
            HIR::Alternation(vec![
                HIR::Sequence("a"),
                HIR::Repetition {
                    regex: Box::new(HIR::Sequence("bc")),
                    min: 0,
                    max: None
                },
                HIR::Sequence("def"),
            ])
        );
    }

    #[test]
    fn parse_alternatives_nested() {
        let hir = parse_regex("a|(b|c)").expect("Failed to parse");
        assert_eq!(
            hir,
            HIR::Alternation(vec![
                HIR::Sequence("a"),
                HIR::Sequence("b"),
                HIR::Sequence("c"),
            ])
        );
    }

    #[test]
    fn parse_concatenation() {
        let hir = parse_regex("ab_cd").expect("Failed to parse");
        assert_eq!(
            hir,
            HIR::Concatenation(vec![HIR::Sequence("ab"), HIR::AnyChar, HIR::Sequence("cd"),])
        );
    }

    #[test]
    fn parse_subregex() {
        let hir = parse_regex("<category>").expect("Failed to parse");
        assert_eq!(hir, HIR::SubRegex("category"));
    }

    #[test]
    fn parse_regex_1() {
        let hir = parse_regex("-?(([123456789]<0-9>*)|0)(.<0-9>+)?([eE][+-]?<0-9>+)?")
            .expect("Failed to parse");
        assert_eq!(
            hir,
            HIR::Concatenation(vec![
                HIR::Repetition {
                    regex: Box::new(HIR::Sequence("-")),
                    min: 0,
                    max: Some(1),
                },
                HIR::Alternation(vec![
                    HIR::Concatenation(vec![
                        HIR::Set(vec![
                            SetMember::Character('1'),
                            SetMember::Character('2'),
                            SetMember::Character('3'),
                            SetMember::Character('4'),
                            SetMember::Character('5'),
                            SetMember::Character('6'),
                            SetMember::Character('7'),
                            SetMember::Character('8'),
                            SetMember::Character('9'),
                        ]),
                        HIR::Repetition {
                            regex: Box::new(HIR::SubRegex("0-9")),
                            min: 0,
                            max: None
                        },
                    ]),
                    HIR::Sequence("0"),
                ]),
                HIR::Repetition {
                    regex: Box::new(HIR::Concatenation(vec![
                        HIR::Sequence("."),
                        HIR::Repetition {
                            regex: Box::new(HIR::SubRegex("0-9")),
                            min: 1,
                            max: None
                        },
                    ])),
                    min: 0,
                    max: Some(1)
                },
                // "([eE][+-]?<0-9>+)?")
                HIR::Repetition {
                    regex: Box::new(HIR::Concatenation(vec![
                        HIR::Set(vec![SetMember::Character('e'), SetMember::Character('E')]),
                        HIR::Repetition {
                            regex: Box::new(HIR::Set(vec![
                                SetMember::Character('+'),
                                SetMember::Character('-')
                            ])),
                            min: 0,
                            max: Some(1)
                        },
                        HIR::Repetition {
                            regex: Box::new(HIR::SubRegex("0-9")),
                            min: 1,
                            max: None
                        },
                    ])),
                    min: 0,
                    max: Some(1)
                },
            ])
        );
    }

    #[test]
    fn parse_category_characters() {
        let hir = parse_category("abcd").expect("Failed to parse");

        assert_eq!(
            hir,
            HIR::Set(vec![
                SetMember::Character('a'),
                SetMember::Character('b'),
                SetMember::Character('c'),
                SetMember::Character('d'),
            ])
        );
    }

    #[test]
    fn parse_category_categories() {
        let hir = parse_category("<cat><dog>").expect("Failed to parse");

        assert_eq!(
            hir,
            HIR::Set(vec![SetMember::Category("cat"), SetMember::Category("dog"),])
        );
    }

    #[test]
    fn parse_category_1() {
        let hir = parse_category("ab<cat>cd<dog>").expect("Failed to parse");

        assert_eq!(
            hir,
            HIR::Set(vec![
                SetMember::Character('a'),
                SetMember::Character('b'),
                SetMember::Category("cat"),
                SetMember::Character('c'),
                SetMember::Character('d'),
                SetMember::Category("dog"),
            ])
        );
    }
}
*/
