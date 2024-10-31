use std::collections::HashMap;

use crate::{
    css::css::{CSSValue, Stylesheet},
    html::dom::{Node, NodeType},
};

#[derive(Debug, PartialEq)]
pub struct StyledNode<'a> {
    pub node_type: &'a NodeType,
    pub children: Vec<StyledNode<'a>>,
    pub properties: HashMap<String, CSSValue>,
}

pub fn to_styled_node<'a>(node: &'a Box<Node>, stylesheet: &Stylesheet) -> Option<StyledNode<'a>> {
    let properties: HashMap<String, CSSValue> = stylesheet
        .rules
        .iter()
        .filter(|rule| rule.matches(node))
        .flat_map(|rule| {
            rule.declarations
                .iter()
                .map(|declaration| (declaration.name.clone(), declaration.value.clone()))
        })
        .collect();

    let children = node
        .children
        .iter()
        .filter_map(|x| to_styled_node(x, stylesheet))
        .collect();

    Some(StyledNode {
        node_type: &node.node_type,
        children,
        properties,
    })
}

#[cfg(test)]
mod tests {

    use rstest::rstest;

    use crate::{
        css::css::{AttributeSelectorOp, Declaration, Rule, SimpleSelector},
        html::dom::Element,
    };

    use super::*;

    #[rstest]
    #[case(
        Stylesheet::new(vec![Rule {
        selectors: vec![SimpleSelector::UniversalSelector],
        declarations: vec![Declaration {
            name: "display".to_string(),
            value: CSSValue::Keyword("block".to_string()),
        }],
        }]),
        vec![(
            "display".to_string(),
            CSSValue::Keyword("block".to_string()),
        )]
    )]
    #[case(
        Stylesheet::new(vec![Rule {
            selectors: vec![SimpleSelector::TypeSelector {
                tag_name: "div".into(),
            }],
            declarations: vec![Declaration {
                name: "display".to_string(),
                value: CSSValue::Keyword("block".to_string()),
            }],
        }]),
        vec![]
    )]
    #[case(
        Stylesheet::new(vec![
            Rule {
                selectors: vec![SimpleSelector::UniversalSelector],
                declarations: vec![Declaration {
                    name: "display".to_string(),
                    value: CSSValue::Keyword("block".into()),
                }],
            },
            Rule {
                selectors: vec![SimpleSelector::TypeSelector {
                    tag_name: "div".into(),
                }],
                declarations: vec![Declaration {
                    name: "display".into(),
                    value: CSSValue::Keyword("inline".into()),
                }],
            },
        ]),
        vec![(
            "display".to_string(),
            CSSValue::Keyword("block".to_string()),
        )]
    )]
    #[case(
        Stylesheet::new(vec![
            Rule {
                selectors: vec![SimpleSelector::UniversalSelector],
                declarations: vec![Declaration {
                    name: "display".to_string(),
                    value: CSSValue::Keyword("block".into()),
                }],
            },
            Rule {
                selectors: vec![SimpleSelector::TypeSelector {
                    tag_name: "p".into(),
                }],
                declarations: vec![
                    Declaration {
                        name: "display".into(),
                        value: CSSValue::Keyword("inline".into()),
                    },
                    Declaration {
                        name: "testname".into(),
                        value: CSSValue::Keyword("testvalue".into()),
                    },
                ],
            },
        ]),
        vec![
            (
                "display".to_string(),
                CSSValue::Keyword("inline".to_string()),
            ),
            (
                "testname".to_string(),
                CSSValue::Keyword("testvalue".to_string()),
            ),
        ]
    )]
    #[case(
        Stylesheet::new(vec![
            Rule {
                selectors: vec![SimpleSelector::UniversalSelector],
                declarations: vec![Declaration {
                    name: "display".to_string(),
                    value: CSSValue::Keyword("block".into()),
                }],
            },
            Rule {
                selectors: vec![SimpleSelector::AttributeSelector {
                    tag_name: "p".into(),
                    op: AttributeSelectorOp::Eq,
                    attribute: "id".into(),
                    value: "hello".into(),
                }],
                declarations: vec![Declaration {
                    name: "testname".into(),
                    value: CSSValue::Keyword("testvalue".into()),
                }],
            },
        ]),
        vec![(
            "display".to_string(),
            CSSValue::Keyword("block".to_string()),
        )]
    )]
    #[case(
        Stylesheet::new(vec![
            Rule {
                selectors: vec![SimpleSelector::UniversalSelector],
                declarations: vec![Declaration {
                    name: "display".to_string(),
                    value: CSSValue::Keyword("block".into()),
                }],
            },
            Rule {
                selectors: vec![SimpleSelector::AttributeSelector {
                    tag_name: "p".into(),
                    op: AttributeSelectorOp::Eq,
                    attribute: "id".into(),
                    value: "test".into(),
                }],
                declarations: vec![Declaration {
                    name: "testname".into(),
                    value: CSSValue::Keyword("testvalue".into()),
                }],
            },
        ]),
        vec![
            (
                "display".to_string(),
                CSSValue::Keyword("block".to_string()),
            ),
            (
                "testname".to_string(),
                CSSValue::Keyword("testvalue".to_string()),
            ),
        ]
    )]
    fn test_to_styled_node_single(
        #[case] stylesheet: Stylesheet,
        #[case] properties: Vec<(String, CSSValue)>,
    ) {
        let e = &Element::new(
            "p".to_string(),
            [("id".to_string(), "test".to_string())]
                .iter()
                .cloned()
                .collect(),
            vec![],
        );

        assert_eq!(
            to_styled_node(e, &stylesheet),
            Some(StyledNode {
                node_type: &e.node_type,
                properties: properties.iter().cloned().collect(),
                children: vec![],
            })
        )
    }

    #[rstest]
    #[case(
        Stylesheet::new(vec![Rule {
            selectors: vec![SimpleSelector::UniversalSelector],
            declarations: vec![Declaration {
                name: "display".to_string(),
                value: CSSValue::Keyword("block".to_string()),
            }],
        }]),
        vec![(
            "display".to_string(),
            CSSValue::Keyword("block".to_string()),
        )]
    )]
    #[case(
        Stylesheet::new(vec![Rule {
            selectors: vec![SimpleSelector::TypeSelector {
                tag_name: "p".into(),
            }],
            declarations: vec![Declaration {
                name: "display".to_string(),
                value: CSSValue::Keyword("block".to_string()),
            }],
        }]),
        vec![]
    )]
    fn test_to_styled_node_nested(
        #[case] stylesheet: Stylesheet,
        #[case] properties: Vec<(String, CSSValue)>,
    ) {
        let parent = &Element::new(
            "div".to_string(),
            [("id".to_string(), "test".to_string())]
                .iter()
                .cloned()
                .collect(),
            vec![Element::new(
                "p".to_string(),
                [("id".to_string(), "test".to_string())]
                    .iter()
                    .cloned()
                    .collect(),
                vec![],
            )],
        );
        let child_node_type = Element::new(
            "p".to_string(),
            [("id".to_string(), "test".to_string())]
                .iter()
                .cloned()
                .collect(),
            vec![],
        )
        .node_type;

        assert_eq!(
            to_styled_node(parent, &stylesheet),
            Some(StyledNode {
                node_type: &parent.node_type,
                properties: properties.iter().cloned().collect(),
                children: vec![StyledNode {
                    node_type: &child_node_type,
                    properties: [(
                        "display".to_string(),
                        CSSValue::Keyword("block".to_string()),
                    )]
                    .iter()
                    .cloned()
                    .collect(),
                    children: vec![],
                }],
            })
        );
    }
}
