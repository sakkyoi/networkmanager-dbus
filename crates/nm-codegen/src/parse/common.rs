use scraper::{ElementRef, Html, Selector};

use crate::model::common::Version;

pub fn find_refentry<'a>(doc: &'a Html) -> Option<ElementRef<'a>> {
    first_with_class_in_html(doc, "refentry")
}

pub fn first_with_class_in_html<'a>(doc: &'a Html, class_name: &str) -> Option<ElementRef<'a>> {
    let selector = Selector::parse(&format!(".{class_name}")).unwrap();
    doc.select(&selector).next()
}

pub fn fist_with_class<'a>(root: &'a ElementRef<'a>, class_name: &str) -> Option<ElementRef<'a>> {
    let selector = Selector::parse(&format!(".{class_name}")).unwrap();
    root.select(&selector).next()
}

pub fn all_with_class<'a>(root: &'a ElementRef<'a>, class_name: &str) -> Vec<ElementRef<'a>> {
    let selector = Selector::parse(&format!(".{class_name}")).unwrap();
    root.select(&selector).collect()
}

pub fn direct_children<'a>(root: &'a ElementRef<'a>) -> Vec<ElementRef<'a>> {
    root.children().filter_map(ElementRef::wrap).collect()
}

pub fn direct_children_with_class<'a>(
    root: &'a ElementRef<'a>,
    class_name: &str,
) -> Vec<ElementRef<'a>> {
    direct_children(root)
        .into_iter()
        .filter(|el| has_class(el, class_name))
        .collect()
}

pub fn direct_children_with_any_ref_class<'a>(root: &'a ElementRef<'a>) -> Vec<ElementRef<'a>> {
    direct_children(root)
        .into_iter()
        .filter(has_any_ref_class)
        .collect()
}

pub fn first_direct_child_with_class<'a>(
    root: &'a ElementRef<'a>,
    class_name: &str,
) -> Option<ElementRef<'a>> {
    direct_children(root)
        .into_iter()
        .find(|el| has_class(el, class_name))
}

pub fn element_classes(el: &ElementRef<'_>) -> Vec<String> {
    el.value().classes().map(ToString::to_string).collect()
}

pub fn has_class(el: &ElementRef<'_>, class_name: &str) -> bool {
    el.value().classes().any(|c| c == class_name)
}

pub fn has_any_ref_class(el: &ElementRef<'_>) -> bool {
    el.value().classes().any(|c| c.starts_with("ref"))
}

pub fn first_heading<'a>(root: &'a ElementRef<'a>) -> Option<ElementRef<'a>> {
    for tag in ["h1", "h2", "h3", "h4", "h5", "h6"] {
        if let Some(el) = first_tag(root, tag) {
            return Some(el);
        }
    }
    None
}

pub fn first_heading_text(root: &ElementRef<'_>) -> Option<String> {
    first_heading(root).map(|h| normalize_text(&h.text().collect::<String>()))
}

pub fn first_tag<'a>(root: &'a ElementRef<'a>, tag_name: &str) -> Option<ElementRef<'a>> {
    let selector = Selector::parse(tag_name).unwrap();
    root.select(&selector).next()
}

pub fn first_tag_text(root: &ElementRef<'_>, tag_name: &str) -> Option<String> {
    first_tag(root, tag_name).map(|el| normalize_text(&el.text().collect::<String>()))
}

pub fn find_anchor_name_before_stop_tags(
    root: &ElementRef<'_>,
    stop_tags: &[&str],
) -> Option<String> {
    for child in root.children() {
        let Some(el) = ElementRef::wrap(child) else {
            continue;
        };

        if el.value().name() == "a" {
            if let Some(name) = el.value().attr("name") {
                return Some(name.to_string());
            }
        }

        if stop_tags.iter().any(|tag| *tag == el.value().name()) {
            break;
        }
    }

    None
}

pub fn find_anchor_name_before_heading(root: &ElementRef<'_>) -> Option<String> {
    find_anchor_name_before_stop_tags(root, &["h1", "h2", "h3", "h4", "h5", "h6"])
}

pub fn collect_doc_lines_after_heading(heading: &ElementRef<'_>) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = heading.next_sibling();

    while let Some(node) = current {
        current = node.next_sibling();

        let Some(el) = ElementRef::wrap(node) else {
            continue;
        };

        if has_any_ref_class(&el) {
            break;
        }

        let tag = el.value().name();
        if matches!(tag, "p" | "ul" | "ol") {
            lines.extend(extract_doc_lines_from_node(&el));
        }
    }

    trim_trailing_blank_lines(&mut lines);
    lines
}

pub fn collect_doc_lines_from_container(container: ElementRef<'_>) -> Vec<String> {
    let mut lines = Vec::new();

    for child in container.children() {
        let Some(el) = ElementRef::wrap(child) else {
            continue;
        };

        let tag = el.value().name();
        if matches!(tag, "p" | "ul" | "ol") {
            lines.extend(extract_doc_lines_from_node(&el));
        }
    }

    trim_trailing_blank_lines(&mut lines);
    lines
}

fn extract_doc_lines_from_node(node: &ElementRef<'_>) -> Vec<String> {
    match node.value().name() {
        "p" => {
            let text = normalize_text(&node.text().collect::<String>());
            if text.is_empty() {
                vec![]
            } else {
                vec![text, String::new()]
            }
        }
        "ul" => {
            let mut lines = Vec::new();

            for child in node.children() {
                let Some(el) = ElementRef::wrap(child) else {
                    continue;
                };

                if el.value().name() != "li" {
                    continue;
                }

                let text = normalize_text(&el.text().collect::<String>());
                if !text.is_empty() {
                    lines.push(String::new());
                }
            }

            if !lines.is_empty() {
                lines.push(String::new());
            }

            lines
        }
        "ol" => {
            let mut lines = Vec::new();
            let mut index = 1;

            for child in node.children() {
                let Some(el) = ElementRef::wrap(child) else {
                    continue;
                };

                if el.value().name() != "li" {
                    continue;
                }

                let text = normalize_text(&el.text().collect::<String>());
                if !text.is_empty() {
                    lines.push(format!("{}. {}", index, text));
                    index += 1;
                }
            }

            if !lines.is_empty() {
                lines.push(String::new());
            }

            lines
        }
        _ => vec![],
    }
}

pub fn extract_since_and_deprecated(lines: &[String]) -> (Option<Version>, Option<Version>) {
    let joined = lines.join("\n");
    let since = extract_version_after(&joined, "Since:");
    let deprecated = extract_version_after(&joined, "Deprecated:");
    (since, deprecated)
}

fn extract_version_after(text: &str, marker: &str) -> Option<Version> {
    let idx = text.find(marker)?;
    let tail = text[idx + marker.len()..].trim_start();

    let mut raw = String::new();
    for ch in tail.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            raw.push(ch);
        } else {
            break;
        }
    }

    parse_version(&raw)
}

fn parse_version(raw: &str) -> Option<Version> {
    let mut parts = raw.split(".");
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some(Version { major, minor })
}

fn trim_trailing_blank_lines(lines: &mut Vec<String>) {
    while matches!(lines.last(), Some(last) if last.is_empty()) {
        lines.pop();
    }
}

pub fn normalize_text(input: &str) -> String {
    input
        .replace("\u{a0}", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
