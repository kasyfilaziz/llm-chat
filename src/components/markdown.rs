use dioxus::prelude::*;
use pulldown_cmark::{Parser, Event, Tag};

#[component]
pub fn Markdown(content: String) -> Element {
    let parser = Parser::new(&content);
    
    let mut elements = Vec::new();
    let mut current_tag: Option<Tag> = None;

    // A very simple state machine to map cmark events to Dioxus RSX
    for event in parser {
        match event {
            Event::Start(tag) => {
                current_tag = Some(tag);
            }
            Event::End(_) => {
                current_tag = None;
            }
            Event::Text(text) => {
                let text_str = text.to_string();
                match &current_tag {
                    Some(Tag::Heading { level, .. }) => {
                        let class = match level {
                            pulldown_cmark::HeadingLevel::H1 => "text-3xl font-bold my-4 text-slate-900",
                            pulldown_cmark::HeadingLevel::H2 => "text-2xl font-bold my-3 text-slate-800",
                            pulldown_cmark::HeadingLevel::H3 => "text-xl font-bold my-2 text-slate-800",
                            _ => "text-lg font-bold my-1 text-slate-800",
                        };
                        elements.push(rsx! { div { class: "{class}", "{text_str}" } });
                    }
                    Some(Tag::Strong) => {
                        elements.push(rsx! { span { class: "font-bold text-slate-900", "{text_str}" } });
                    }
                    Some(Tag::Emphasis) => {
                        elements.push(rsx! { span { class: "italic", "{text_str}" } });
                    }
                    Some(Tag::CodeBlock(_)) => {
                        elements.push(rsx! { 
                            pre { class: "markdown-code-block",
                                code { "{text_str}" }
                            }
                        });
                    }
                    None => {
                        elements.push(rsx! { p { class: "mb-3 last:mb-0", "{text_str}" } });
                    }
                    _ => {
                        elements.push(rsx! { span { "{text_str}" } });
                    }
                }
            }
            Event::Code(code) => {
                elements.push(rsx! { code { class: "bg-slate-100 border border-slate-200 px-1.5 py-0.5 rounded text-pink-600 font-mono text-xs", "{code}" } });
            }
            Event::SoftBreak => {
                // Soft breaks can be treated as a space in simple markdown rendering
                elements.push(rsx! { " " });
            }
            _ => {}
        }
    }

    rsx! {
        div { class: "markdown-content leading-relaxed",
            {elements.into_iter()}
        }
    }
}
