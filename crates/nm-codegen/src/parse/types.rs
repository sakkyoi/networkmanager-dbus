use anyhow::{Result, anyhow};
use scraper::{ElementRef, Html, Selector};

use crate::model::{
    common::Version,
    types::{EnumDef, EnumValue, TypesPage},
};

pub fn parse_types_page(html: &str, base_url: &str) -> Result<TypesPage> {
    let doc = Html::parse_document(html);

    let section_sel = Selector::parse(".refsect2").unwrap();
    let h2_sel = Selector::parse("h2").unwrap();
    let h3_sel = Selector::parse("h3").unwrap();
    let refsect3_sel = Selector::parse(".refsect3").unwrap();
    let h4_sel = Selector::parse("h4").unwrap();
    let row_sel = Selector::parse("tbody tr").unwrap();
    let name_sel = Selector::parse(".enum_member_name p").unwrap();
    let value_sel = Selector::parse(".enum_member_value code").unwrap();
    let desc_cell_sel = Selector::parse(".enum_member_description").unwrap();

    let page_description = doc
        .select(&h2_sel)
        .next()
        .map(|h2| collect_doc_lines_after_heading(&h2))
        .unwrap_or_default();

    let mut enums = Vec::new();

    for section in doc.select(&section_sel) {
        let Some(h3) = section.select(&h3_sel).next() else {
            continue;
        };

        let title = normalize_text(&h3.text().collect::<String>());

        let Some(enum_name) = title.strip_prefix("enum ") else {
            continue;
        };

        let description = collect_doc_lines_after_h3(&h3);
        let (enum_since, enum_deprecated) = extract_since_and_deprecated(&description);

        let values_section = find_values_section(&section, &refsect3_sel, &h4_sel);
        let Some(values_section) = values_section else {
            continue;
        };

        let source_url = find_enum_anchor(&section)
            .map(|anchor| format!("{base_url}#{anchor}"))
            .or_else(|| Some(base_url.to_string()));

        let mut values = Vec::new();

        for row in values_section.select(&row_sel) {
            let name = row
                .select(&name_sel)
                .next()
                .map(|n| normalize_text(&n.text().collect::<String>()))
                .unwrap_or_default();

            if name.is_empty() {
                continue;
            }

            let value = row
                .select(&value_sel)
                .next()
                .map(|v| normalize_text(&v.text().collect::<String>()))
                .ok_or_else(|| anyhow!("missing enum value code for {}", name))?;

            let description = row
                .select(&desc_cell_sel)
                .next()
                .map(collect_doc_lines_from_container)
                .unwrap_or_default();

            let (since, deprecated) = extract_since_and_deprecated(&description);

            values.push(EnumValue {
                name,
                value,
                description,
                since,
                deprecated,
            });
        }

        if !values.is_empty() {
            enums.push(EnumDef {
                name: enum_name.to_string(),
                description,
                values,
                source_url,
                since: enum_since,
                deprecated: enum_deprecated,
            });
        }
    }

    Ok(TypesPage {
        description: page_description,
        source_url: Some(base_url.to_string()),
        enums,
    })
}

fn find_values_section<'a>(
    section: &'a ElementRef<'a>,
    refsect3_sel: &Selector,
    h4_sel: &Selector,
) -> Option<ElementRef<'a>> {
    for sub in section.select(refsect3_sel) {
        let Some(h4) = sub.select(h4_sel).next() else {
            continue;
        };

        let title = normalize_text(&h4.text().collect::<String>());
        if title == "Values" {
            return Some(sub);
        }
    }

    None
}

fn find_enum_anchor(section: &ElementRef<'_>) -> Option<String> {
    for child in section.children() {
        let Some(el) = ElementRef::wrap(child) else {
            continue;
        };

        if el.value().name() == "a" {
            if let Some(name) = el.value().attr("name") {
                return Some(name.to_string());
            }
        }
    }

    None
}

fn collect_doc_lines_after_heading(heading: &ElementRef<'_>) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = heading.next_sibling();

    while let Some(node) = current {
        current = node.next_sibling();

        let Some(el) = ElementRef::wrap(node) else {
            continue;
        };

        let class_list = el.value().classes().collect::<Vec<_>>();
        if class_list
            .iter()
            .any(|c| *c == "refsect2" || *c == "refsect3")
        {
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

fn collect_doc_lines_after_h3(h3: &ElementRef<'_>) -> Vec<String> {
    let mut lines = Vec::new();

    let mut next = h3.next_sibling();
    while let Some(node) = next {
        if let Some(el) = ElementRef::wrap(node) {
            let class_list = el.value().classes().collect::<Vec<_>>();
            if class_list
                .iter()
                .any(|c| *c == "refsect3" || *c == "refsect2")
            {
                break;
            }

            let tag = el.value().name();
            if matches!(tag, "p" | "ul" | "ol") {
                lines.extend(extract_doc_lines_from_node(&el));
            }
        }

        next = node.next_sibling();
    }

    trim_trailing_blank_lines(&mut lines);
    lines
}

fn collect_doc_lines_from_container(container: ElementRef<'_>) -> Vec<String> {
    let mut lines = Vec::new();

    for child in container.children() {
        if let Some(el) = ElementRef::wrap(child) {
            let tag = el.value().name();
            if matches!(tag, "p" | "ul" | "ol") {
                lines.extend(extract_doc_lines_from_node(&el));
            }
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
            let li_sel = Selector::parse("li").unwrap();
            let mut lines = Vec::new();

            for li in node.select(&li_sel) {
                let text = normalize_text(&li.text().collect::<String>());
                if !text.is_empty() {
                    lines.push(format!("- {}", text));
                }
            }

            if !lines.is_empty() {
                lines.push(String::new());
            }

            lines
        }
        "ol" => {
            let li_sel = Selector::parse("li").unwrap();
            let mut lines = Vec::new();

            for (i, li) in node.select(&li_sel).enumerate() {
                let text = normalize_text(&li.text().collect::<String>());
                if !text.is_empty() {
                    lines.push(format!("{}. {}", i + 1, text));
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

fn extract_since_and_deprecated(lines: &[String]) -> (Option<Version>, Option<Version>) {
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

fn normalize_text(input: &str) -> String {
    input
        .replace("\u{a0}", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
