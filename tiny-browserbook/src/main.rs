use std::rc::Rc;

use tiny_browserbook::{
    css::css,
    html::{
        dom::{Node, NodeType},
        html::parse,
    },
    layout::layout::to_layout_box,
    render::render::to_element_container,
    renderer::renderer::Renderer,
    style::style::to_styled_node,
};

const HTML: &str = r#"<body>
    <p>hello</p>
    <p class="inline">world</p>
    <p class="inline">:)</p>
    <div class="none"><p>this should not be shown</p></div>
    <style>
        .none { 
            display: none;
        }
        .inline {
            display: inline;
        }
    </style>

    <div id="result">
        <p>not loaded</p>
    </div
    <script>
        document.getElementById("result").innerHTML = `\x3cp\x3eloaded\x3c/p\x3e`
    </script> 
</body>"#;

const DEFAULT_STYLESHEET: &str = r#"
script, style {
    display: none;
}
p, div {
    display: block;
}
"#;

fn collect_tag_inners(node: &Box<Node>, tag_name: &str) -> Vec<String> {
    if let NodeType::Element(ref element) = node.node_type {
        if element.tag_name.as_str() == tag_name {
            return vec![node.inner_text()];
        }
    }
    node.children
        .iter()
        .map(|child| collect_tag_inners(child, tag_name))
        .collect::<Vec<Vec<String>>>()
        .into_iter()
        .flatten()
        .collect()
}

fn main() {
    let mut siv = cursive::default();

    let node = parse(HTML);
    let stylesheet = css::parse(&format!(
        "{}\n{}",
        DEFAULT_STYLESHEET,
        collect_tag_inners(&node, "style".into()).join("\n")
    ));

    let container = to_styled_node(&node, &stylesheet)
        .and_then(|styled_node| Some(to_layout_box(styled_node)))
        .and_then(|layout_box| Some(to_element_container(layout_box)));
    if let Some(c) = container {
        siv.add_fullscreen_layer(c);
    }

    let mut renderer = Renderer::new(Rc::new(siv.cb_sink().clone()), node);
    renderer.execute_inline_scripts();
    siv.add_fullscreen_layer(renderer);

    siv.run();
}
