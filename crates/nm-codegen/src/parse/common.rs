use scraper::{ElementRef, Html, Selector};

use crate::model::common::Version;

#[derive(Debug, Clone)]
pub struct ParsedSectionDetail {
    pub title: String,
    pub source_url: Option<String>,
    pub description: Vec<String>,
    pub since: Option<Version>,
    pub deprecated: Option<Version>,
}

pub fn find_refentry<'a>(doc: &'a Html) -> Option<ElementRef<'a>> {
    let selector = Selector::parse(".refentry").unwrap();
    doc.select(&selector).next()
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

pub fn find_anchor_name_before_heading(root: &ElementRef<'_>) -> Option<String> {
    for child in root.children() {
        let Some(el) = ElementRef::wrap(child) else {
            continue;
        };

        if el.value().name() == "a" {
            if let Some(name) = el.value().attr("name") {
                return Some(name.to_string());
            }
        }

        if matches!(el.value().name(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
            break;
        }
    }

    None
}

pub fn source_url_from_section_anchor(section: &ElementRef<'_>, base_url: &str) -> Option<String> {
    find_anchor_name_before_heading(section)
        .map(|anchor| format!("{base_url}#{anchor}"))
        .or_else(|| Some(base_url.to_string()))
}

pub fn parse_refentry_page_identity(refentry: &ElementRef<'_>) -> (String, Vec<String>) {
    let page_name = direct_children_with_class(refentry, "refnamediv")
        .into_iter()
        .find_map(|div| first_heading_text(&div))
        .or_else(|| first_heading_text(refentry))
        .unwrap_or_default();

    let page_description = direct_children_with_class(refentry, "refnamediv")
        .into_iter()
        .find_map(|div| {
            let heading = first_tag(&div, "h2")?;
            Some(collect_doc_lines_after_heading(&heading))
        })
        .unwrap_or_default();

    (page_name, page_description)
}

pub fn parse_section_detail(
    section: &ElementRef<'_>,
    heading_tag: &str,
    base_url: &str,
) -> Option<ParsedSectionDetail> {
    let heading = first_tag(section, heading_tag)?;
    let title = normalize_text(&heading.text().collect::<String>());
    let description = collect_doc_lines_after_heading(&heading);
    let (since, deprecated) = extract_since_and_deprecated(&description);

    Some(ParsedSectionDetail {
        title,
        source_url: source_url_from_section_anchor(section, base_url),
        description,
        since,
        deprecated,
    })
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
