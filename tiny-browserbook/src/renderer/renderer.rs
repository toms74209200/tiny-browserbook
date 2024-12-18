use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use cursive::{CbSink, View};

use crate::{
    css::css::parse,
    html::dom::{Node, NodeType},
    javascript::{javascript::JavascriptRuntime, renderapi::RendererAPI},
    layout::layout::to_layout_box,
    render::render::{to_element_container, ElementContainer},
    style::style::to_styled_node,
};

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

pub struct Renderer {
    view: ElementContainer,
    document_element: Arc<Mutex<Box<Node>>>,
    js_runtime_instance: JavascriptRuntime,
}

impl Renderer {
    pub fn new(ui_cb_sink: Rc<CbSink>, document_element: Box<Node>) -> Self {
        let stylesheet = parse(&format!(
            "{}\n{}",
            DEFAULT_STYLESHEET,
            collect_tag_inners(&document_element, "style".into()).join("\n")
        ));

        let view = to_styled_node(&document_element, &stylesheet)
            .and_then(|styled_node| Some(to_layout_box(styled_node)))
            .and_then(|layout_box| Some(to_element_container(layout_box)))
            .unwrap();

        let document_element = Arc::new(Mutex::new(document_element));
        let document_element_ref = document_element.clone();
        Self {
            document_element,
            view,
            js_runtime_instance: JavascriptRuntime::new(
                document_element_ref,
                Arc::new(RendererAPI::new(ui_cb_sink)),
            ),
        }
    }

    pub fn rerender(&mut self) {
        let document_element = self.document_element.lock().unwrap();
        let stylesheet = parse(&format!(
            "{}\n{}",
            DEFAULT_STYLESHEET,
            collect_tag_inners(&document_element, "style".into()).join("\n")
        ));
        self.view = to_styled_node(&document_element, &stylesheet)
            .and_then(|styled_node| Some(to_layout_box(styled_node)))
            .and_then(|layout_box| Some(to_element_container(layout_box)))
            .unwrap();
    }

    pub fn execute_inline_scripts(&mut self) {
        let scripts = {
            let document_element = self.document_element.lock().unwrap();
            collect_tag_inners(&document_element, "script".into()).join("\n")
        };
        self.js_runtime_instance
            .execute("(inline)", scripts.as_str())
            .unwrap();
    }
}

impl View for Renderer {
    fn draw(&self, printer: &cursive::Printer) {
        self.view.draw(printer)
    }

    fn layout(&mut self, v: cursive::Vec2) {
        self.view.layout(v)
    }

    fn needs_relayout(&self) -> bool {
        self.view.needs_relayout()
    }

    fn required_size(&mut self, constraint: cursive::Vec2) -> cursive::Vec2 {
        self.view.required_size(constraint)
    }

    fn on_event(&mut self, e: cursive::event::Event) -> cursive::event::EventResult {
        self.view.on_event(e)
    }

    fn call_on_any<'a>(&mut self, s: &cursive::view::Selector<'_>, cb: cursive::event::AnyCb<'a>) {
        self.view.call_on_any(s, cb)
    }

    fn focus_view(
        &mut self,
        s: &cursive::view::Selector<'_>,
    ) -> Result<cursive::event::EventResult, cursive::view::ViewNotFound> {
        self.view.focus_view(s)
    }

    fn take_focus(
        &mut self,
        source: cursive::direction::Direction,
    ) -> Result<cursive::event::EventResult, cursive::view::CannotFocus> {
        self.view.take_focus(source)
    }

    fn important_area(&self, view_size: cursive::Vec2) -> cursive::Rect {
        self.view.important_area(view_size)
    }

    fn type_name(&self) -> &'static str {
        self.view.type_name()
    }
}

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}
