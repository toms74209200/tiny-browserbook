use combine::{
    choice,
    error::StreamError,
    many, many1, optional,
    parser::char::{self, letter, newline, space},
    sep_by, sep_end_by, ParseError, Parser, Stream,
};

use crate::html::dom::{Node, NodeType};

#[derive(Debug, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

impl Stylesheet {
    pub fn new(rules: Vec<Rule>) -> Self {
        Stylesheet { rules }
    }
}

#[derive(Debug, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: CSSValue,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CSSValue {
    Keyword(String),
}

#[derive(Debug, PartialEq)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

impl Rule {
    pub fn matches(&self, n: &Box<Node>) -> bool {
        self.selectors.iter().any(|s| s.matches(n))
    }
}

pub type Selector = SimpleSelector;

#[derive(Debug, PartialEq)]
pub enum SimpleSelector {
    UniversalSelector,
    TypeSelector {
        tag_name: String,
    },
    AttributeSelector {
        tag_name: String,
        op: AttributeSelectorOp,
        attribute: String,
        value: String,
    },
    ClassSelector {
        class_name: String,
    },
}

impl SimpleSelector {
    pub fn matches(&self, n: &Box<Node>) -> bool {
        match self {
            SimpleSelector::UniversalSelector => true,
            SimpleSelector::TypeSelector { tag_name } => match n.node_type {
                NodeType::Element(ref e) => e.tag_name.as_str() == tag_name,
                _ => false,
            },
            SimpleSelector::AttributeSelector {
                tag_name,
                op,
                attribute,
                value,
            } => match n.node_type {
                NodeType::Element(ref e) => e.tag_name.as_str() == tag_name,
                _ => false,
            },
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AttributeSelectorOp {
    Eq,
    Contain,
}

/// Parse CSS stylesheet
/// # Example
/// ```
/// use tiny_browserbook::css::css::parse;
/// let css = r#"
/// test [foo=bar] {
///   aa: bb;
///   cc: dd;
/// }
/// rule {
///   ee: dd;
/// }
/// "#;
/// let result = parse(css);
/// assert_eq!(result.rules.len(), 2);
/// ```
pub fn parse(raw: &str) -> Stylesheet {
    rules()
        .parse(raw)
        .map(|(rules, _)| Stylesheet::new(rules))
        .unwrap()
}

fn whitespaces<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many::<String, _, _>(space().or(newline()))
}

fn rules<Input>() -> impl Parser<Input, Output = Vec<Rule>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (whitespaces(), many(rule().skip(whitespaces()))).map(|(_, rules)| rules)
}

fn rule<Input>() -> impl Parser<Input, Output = Rule>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        selectors().skip(whitespaces()),
        char::char('{').skip(whitespaces()),
        declarations().skip(whitespaces()),
        char::char('}'),
    )
        .map(|(selectors, _, declarations, _)| Rule {
            selectors,
            declarations,
        })
}

fn selectors<Input>() -> impl Parser<Input, Output = Vec<Selector>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_by(
        simple_selector().skip(whitespaces()),
        char::char(',').skip(whitespaces()),
    )
}

fn simple_selector<Input>() -> impl Parser<Input, Output = SimpleSelector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let universal_selector = char::char('*').map(|_| SimpleSelector::UniversalSelector);
    let class_selector = (char::char('.'), many1(letter()))
        .map(|(_, class_name)| SimpleSelector::ClassSelector { class_name });
    let type_or_attribute_selector = (
        many1(letter()).skip(whitespaces()),
        optional((
            char::char('[').skip(whitespaces()),
            many1(letter()),
            choice((char::string("="), char::string("~="))),
            many1(letter()),
            char::char(']'),
        )),
    )
        .and_then(|(tag_name, opts)| match opts {
            Some((_, attribute, op, value, _)) => {
                let op = match op {
                    "=" => AttributeSelectorOp::Eq,
                    "~=" => AttributeSelectorOp::Contain,
                    _ => {
                        return Err(<Input::Error as combine::error::ParseError<
                            char,
                            Input::Range,
                            Input::Position,
                        >>::StreamError::message_static_message(
                            "invalid attribute selector op",
                        ))
                    }
                };
                Ok(SimpleSelector::AttributeSelector {
                    tag_name,
                    op,
                    attribute,
                    value,
                })
            }
            None => Ok(SimpleSelector::TypeSelector { tag_name: tag_name }),
        });
    choice((
        universal_selector,
        class_selector,
        type_or_attribute_selector,
    ))
}

fn declarations<Input>() -> impl Parser<Input, Output = Vec<Declaration>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_end_by(
        declaration().skip(whitespaces()),
        char::char(';').skip(whitespaces()),
    )
}

fn declaration<Input>() -> impl Parser<Input, Output = Declaration>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        many1(letter()).skip(whitespaces()),
        char::char(':').skip(whitespaces()),
        css_value(),
    )
        .map(|(k, _, v)| Declaration { name: k, value: v })
}

fn css_value<Input>() -> impl Parser<Input, Output = CSSValue>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(letter()).map(|s| CSSValue::Keyword(s))
}

#[cfg(test)]
mod tests {
    use crate::html::dom::Element;

    use super::*;

    #[test]
    fn test_rules() {
        assert_eq!(
            rules().parse("test [foo=bar] { aa: bb; cc: dd; } rule { ee: dd; }"),
            Ok((
                vec![
                    Rule {
                        selectors: vec![SimpleSelector::AttributeSelector {
                            tag_name: "test".to_string(),
                            op: AttributeSelectorOp::Eq,
                            attribute: "foo".to_string(),
                            value: "bar".to_string()
                        }],
                        declarations: vec![
                            Declaration {
                                name: "aa".to_string(),
                                value: CSSValue::Keyword("bb".to_string())
                            },
                            Declaration {
                                name: "cc".to_string(),
                                value: CSSValue::Keyword("dd".to_string())
                            }
                        ]
                    },
                    Rule {
                        selectors: vec![SimpleSelector::TypeSelector {
                            tag_name: "rule".to_string()
                        }],
                        declarations: vec![Declaration {
                            name: "ee".to_string(),
                            value: CSSValue::Keyword("dd".to_string())
                        }]
                    }
                ],
                ""
            ))
        );
    }

    #[test]
    fn test_rule() {
        assert_eq!(
            rule().parse("test [foo=bar] {}"),
            Ok((
                Rule {
                    selectors: vec![SimpleSelector::AttributeSelector {
                        tag_name: "test".to_string(),
                        attribute: "foo".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "bar".to_string()
                    }],
                    declarations: vec![]
                },
                ""
            ))
        );
    }

    #[test]
    fn test_rule_multiple() {
        assert_eq!(
            rule().parse("test [foo=bar], testtest[piyo~=guoo] {}"),
            Ok((
                Rule {
                    selectors: vec![
                        SimpleSelector::AttributeSelector {
                            tag_name: "test".to_string(),
                            attribute: "foo".to_string(),
                            op: AttributeSelectorOp::Eq,
                            value: "bar".to_string()
                        },
                        SimpleSelector::AttributeSelector {
                            tag_name: "testtest".to_string(),
                            attribute: "piyo".to_string(),
                            op: AttributeSelectorOp::Contain,
                            value: "guoo".to_string()
                        }
                    ],
                    declarations: vec![]
                },
                ""
            ))
        );
    }

    #[test]
    fn test_rule_value() {
        assert_eq!(
            rule().parse("test [foo=bar] { aa: bb; cc: dd; }"),
            Ok((
                Rule {
                    selectors: vec![SimpleSelector::AttributeSelector {
                        tag_name: "test".to_string(),
                        attribute: "foo".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "bar".to_string()
                    }],
                    declarations: vec![
                        Declaration {
                            name: "aa".to_string(),
                            value: CSSValue::Keyword("bb".to_string())
                        },
                        Declaration {
                            name: "cc".to_string(),
                            value: CSSValue::Keyword("dd".to_string()),
                        }
                    ]
                },
                ""
            ))
        );
    }

    #[test]
    fn test_selectors() {
        assert_eq!(
            selectors().parse("test [foo=bar], a"),
            Ok((
                vec![
                    SimpleSelector::AttributeSelector {
                        tag_name: "test".to_string(),
                        attribute: "foo".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "bar".to_string()
                    },
                    SimpleSelector::TypeSelector {
                        tag_name: "a".to_string(),
                    }
                ],
                ""
            ))
        );
    }

    #[test]
    fn test_simple_selector_universal() {
        assert_eq!(
            simple_selector().parse("*"),
            Ok((SimpleSelector::UniversalSelector, ""))
        );
    }

    #[test]
    fn test_simple_selector_type() {
        assert_eq!(
            simple_selector().parse("test"),
            Ok((
                SimpleSelector::TypeSelector {
                    tag_name: "test".to_string(),
                },
                ""
            ))
        );
    }

    #[test]
    fn test_simple_selector_attribute() {
        assert_eq!(
            simple_selector().parse("test [foo=bar]"),
            Ok((
                SimpleSelector::AttributeSelector {
                    tag_name: "test".to_string(),
                    attribute: "foo".to_string(),
                    op: AttributeSelectorOp::Eq,
                    value: "bar".to_string()
                },
                ""
            ))
        );
    }

    #[test]
    fn test_simple_selector_class() {
        assert_eq!(
            simple_selector().parse(".test"),
            Ok((
                SimpleSelector::ClassSelector {
                    class_name: "test".to_string(),
                },
                ""
            ))
        );
    }

    #[test]
    fn test_declarations() {
        assert_eq!(
            declarations().parse("foo: bar; piyo: piyopiyo;"),
            Ok((
                vec![
                    Declaration {
                        name: "foo".to_string(),
                        value: CSSValue::Keyword("bar".to_string())
                    },
                    Declaration {
                        name: "piyo".to_string(),
                        value: CSSValue::Keyword("piyopiyo".to_string())
                    }
                ],
                ""
            ))
        );
    }

    #[test]
    fn test_universal_selector_behaviour() {
        let e = &Element::new(
            "p".to_string(),
            [
                ("id".to_string(), "test".to_string()),
                ("class".to_string(), "testclass".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            vec![],
        );
        assert_eq!(SimpleSelector::UniversalSelector.matches(e), true);
    }

    #[test]
    fn test_universal_selector_behaviour_with_tag() {
        let e = &Element::new(
            "p".to_string(),
            [
                ("id".to_string(), "test".to_string()),
                ("class".to_string(), "testclass".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            vec![],
        );
        assert_eq!(
            (SimpleSelector::TypeSelector {
                tag_name: "p".into()
            })
            .matches(e),
            true
        );
        assert_eq!(
            (SimpleSelector::TypeSelector {
                tag_name: "invalid".into(),
            })
            .matches(e),
            false
        );
    }

    #[test]
    fn test_attribute_selector_behaviour() {
        let e = &Element::new(
            "p".to_string(),
            [
                ("id".to_string(), "test".to_string()),
                ("class".to_string(), "testclass".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            vec![],
        );

        assert_eq!(
            (SimpleSelector::AttributeSelector {
                tag_name: "p".into(),
                attribute: "id".into(),
                value: "test".into(),
                op: AttributeSelectorOp::Eq,
            })
            .matches(e),
            true
        );
    }
}
