use anyhow::{Result, anyhow};
use scraper::{ElementRef, Html};

use crate::parse::common::normalize_text;

#[derive(Debug, Clone)]
pub struct SpecToc {
    pub entries: Vec<SpecTocEntry>,
}

#[derive(Debug, Clone)]
pub struct SpecTocEntry {
    pub object_path_label: String,
    pub interfaces: Vec<SpecTocInterface>,
}

#[derive(Debug, Clone)]
pub struct SpecTocInterface {
    pub name: String,
    pub href: String,
    pub description: String,
}

pub fn parse_spec_toc(html: &str) -> Result<SpecToc> {
    let doc = Html::parse_document(html);
    let toc = find_first_dl_toc(&doc).ok_or_else(|| anyhow!("failed to find dl.toc"))?;

    let mut entries = Vec::new();

    for (dt, dd) in collect_dt_dd_pairs(toc) {
        let object_path_label = normalize_text(&dt.text().collect::<String>());

        let interfaces = find_first_nested_dl(&dd)
            .map(parse_interface_list_dl)
            .unwrap_or_default();

        entries.push(SpecTocEntry {
            object_path_label,
            interfaces,
        });
    }

    Ok(SpecToc { entries })
}

fn parse_interface_list_dl(dl: ElementRef<'_>) -> Vec<SpecTocInterface> {
    let mut interfaces = Vec::new();

    for child in dl.children() {
        let Some(dt) = ElementRef::wrap(child) else {
            continue;
        };

        if dt.value().name() != "dt" {
            continue;
        }

        let (name, href) = extract_refentrytitle(&dt);
        if name.is_empty() {
            continue;
        }

        let description = extract_refpurpose(&dt);

        interfaces.push(SpecTocInterface {
            name,
            href,
            description,
        });
    }

    interfaces
}

fn find_first_dl_toc(doc: &Html) -> Option<ElementRef<'_>> {
    for node in doc.tree.root().descendants() {
        let Some(el) = ElementRef::wrap(node) else {
            continue;
        };

        if el.value().name() != "dl" {
            continue;
        }

        let classes: Vec<_> = el.value().classes().collect();
        if classes.iter().any(|c| *c == "toc") {
            return Some(el);
        }
    }

    None
}

fn find_first_nested_dl<'a>(root: &ElementRef<'a>) -> Option<ElementRef<'a>> {
    for child in root.children() {
        let Some(el) = ElementRef::wrap(child) else {
            continue;
        };

        if el.value().name() == "dl" {
            return Some(el);
        }

        for desc in el.children() {
            let Some(desc_el) = ElementRef::wrap(desc) else {
                continue;
            };

            if desc_el.value().name() == "dl" {
                return Some(desc_el);
            }
        }
    }

    None
}

fn collect_dt_dd_pairs(dl: ElementRef<'_>) -> Vec<(ElementRef<'_>, ElementRef<'_>)> {
    let mut pairs = Vec::new();
    let mut current_dt: Option<ElementRef<'_>> = None;

    for child in dl.children() {
        let Some(el) = ElementRef::wrap(child) else {
            continue;
        };

        match el.value().name() {
            "dt" => current_dt = Some(el),
            "dd" => {
                if let Some(dt) = current_dt.take() {
                    pairs.push((dt, el));
                }
            }
            _ => {}
        }
    }

    pairs
}

fn extract_refentrytitle(dt: &ElementRef<'_>) -> (String, String) {
    for child in dt.children() {
        let Some(el) = ElementRef::wrap(child) else {
            continue;
        };

        let classes: Vec<_> = el.value().classes().collect();
        if classes.iter().any(|c| *c == "refentrytitle") {
            let name = normalize_text(&el.text().collect::<String>());
            let href = find_first_anchor_href(&el).unwrap_or_default();
            return (name, href);
        }
    }

    (String::new(), String::new())
}

fn extract_refpurpose(dt: &ElementRef<'_>) -> String {
    for child in dt.children() {
        let Some(el) = ElementRef::wrap(child) else {
            continue;
        };

        let classes: Vec<_> = el.value().classes().collect();
        if classes.iter().any(|c| *c == "refpurpose") {
            let raw = normalize_text(&el.text().collect::<String>());
            // I'm not sure if there is a leading space or not so we trim both of those two cases
            return raw
                .trim_start_matches(" — ")
                .trim_start_matches("— ")
                .trim_start()
                .to_string();
        }
    }

    String::new()
}

fn find_first_anchor_href(root: &ElementRef<'_>) -> Option<String> {
    for child in root.children() {
        let Some(el) = ElementRef::wrap(child) else {
            continue;
        };

        if el.value().name() == "a" {
            return el.value().attr("href").map(ToString::to_string);
        }
    }

    None
}

