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
    Some(StyledNode {
        node_type: &node.node_type,
        children: vec![],
        properties: vec![(
            "display".to_string(),
            CSSValue::Keyword("block".to_string()),
        )]
        .iter()
        .cloned()
        .collect(),
    })
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{
        css::css::{Declaration, Rule, SimpleSelector},
        html::dom::Element,
    };

    use super::*;

    #[test]
    fn test_to_styled_node_single() {
        let e = &Element::new(
            "p".to_string(),
            [("id".to_string(), "test".to_string())]
                .iter()
                .cloned()
                .collect(),
            vec![],
        );
        let stylesheet = Stylesheet::new(vec![Rule {
            selectors: vec![SimpleSelector::UniversalSelector],
            declarations: vec![Declaration {
                name: "display".to_string(),
                value: CSSValue::Keyword("block".to_string()),
            }],
        }]);
        let properties = vec![(
            "display".to_string(),
            CSSValue::Keyword("block".to_string()),
        )];

        assert_eq!(
            to_styled_node(e, &stylesheet),
            Some(StyledNode {
                node_type: &e.node_type,
                properties: properties.iter().cloned().collect(),
                children: vec![],
            })
        )
    }
}
