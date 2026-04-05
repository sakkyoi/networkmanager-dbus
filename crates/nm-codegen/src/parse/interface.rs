use anyhow::Result;
use scraper::{ElementRef, Html, Selector};

use crate::{
    model::{
        interface::{
            ArgDirection, ArgumentDef, InterfaceDef, MethodDef, PropertyAccess, PropertyDef, SignalDef,
        },
    },
    parse::common::{
        collect_doc_lines_after_heading, direct_children_with_class, extract_since_and_deprecated,
        find_anchor_name_before_heading, find_refentry, first_heading_text, first_tag, normalize_text,
        collect_doc_lines_from_container,
    },
};

pub fn parse_interface_page(html: &str, base_url: &str) -> Result<InterfaceDef> {
    let doc = Html::parse_document(html);
    let Some(refentry) = find_refentry(&doc) else {
        return Ok(InterfaceDef {
            source_url: Some(base_url.to_string()),
            ..Default::default()
        });
    };

    let (page_name, page_description) = parse_page_identity(&refentry);
    let (page_since, page_deprecated) = extract_since_and_deprecated(&page_description);

    let mut methods = Vec::new();
    let mut properties = Vec::new();
    let mut signals = Vec::new();

    for section in direct_children_with_class(&refentry, "refsect1") {
        let Some(title) = first_heading_text(&section) else {
            continue;
        };

        match title.as_str() {
            "Method Details" => {
                for item in direct_children_with_class(&section, "refsect2") {
                    if let Some(method) = parse_method_detail(&item, base_url) {
                        methods.push(method);
                    }
                }
            }
            "Property Details" => {
                for item in direct_children_with_class(&section, "refsect2") {
                    if let Some(property) = parse_property_detail(&item, base_url) {
                        properties.push(property);
                    }
                }
            }
            "Signal Details" => {
                for item in direct_children_with_class(&section, "refsect2") {
                    if let Some(signal) = parse_signal_detail(&item, base_url) {
                        signals.push(signal);
                    }
                }
            }
            _ => {},
        }
    }

    Ok(InterfaceDef {
        name: page_name,
        source_url: Some(base_url.to_string()),
        description: page_description,
        since: page_since,
        deprecated: page_deprecated,
        object_path_family_label: None,
        object_path_pattern: None,
        methods,
        properties,
        signals,
    })
}

fn parse_page_identity(refentry: &ElementRef<'_>) -> (String, Vec<String>) {
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

fn parse_method_detail(section: &ElementRef<'_>, base_url: &str) -> Option<MethodDef> {
    let heading = first_tag(section, "h3")?;
    let raw_name = normalize_text(&heading.text().collect::<String>());
    let name = normalize_member_title(&raw_name, "method");
    if name.is_empty() {
        return None;
    }

    let source_url = find_anchor_name_before_heading(section)
        .map(|anchor| format!("{base_url}#{anchor}"))
        .or_else(|| Some(base_url.to_string()));

    let description = collect_doc_lines_after_heading(&heading);
    let (since, deprecated) = extract_since_and_deprecated(&description);
    let (inputs, outputs) = parse_method_args(section);

    Some(MethodDef {
        name,
        source_url,
        description,
        since,
        deprecated,
        inputs,
        outputs,
    })
}

fn parse_signal_detail(section: &ElementRef<'_>, base_url: &str) -> Option<SignalDef> {
    let heading = first_tag(section, "h3")?;
    let raw_name = normalize_text(&heading.text().collect::<String>());
    let name = normalize_member_title(&raw_name, "signal");

    if name.is_empty() {
        return None;
    }

    let source_url = find_anchor_name_before_heading(section)
        .map(|anchor| format!("{base_url}#{anchor}"))
        .or_else(|| Some(base_url.to_string()));

    let description = collect_doc_lines_after_heading(&heading);
    let (since, deprecated) = extract_since_and_deprecated(&description);
    let args = parse_signal_args(section);

    Some(SignalDef {
        name,
        source_url,
        description,
        since,
        deprecated,
        args,
    })
}

fn parse_property_detail(section: &ElementRef<'_>, base_url: &str) -> Option<PropertyDef> {
    let heading = first_tag(section, "h3")?;
    let raw_name = normalize_text(&heading.text().collect::<String>());
    let name = normalize_member_title(&raw_name, "property");

    if name.is_empty() {
        return None;
    }

    let source_url = find_anchor_name_before_heading(section)
        .map(|anchor| format!("{base_url}#{anchor}"))
        .or_else(|| Some(base_url.to_string()));

    let description = collect_doc_lines_after_heading(&heading);
    let (since, deprecated) = extract_since_and_deprecated(&description);

    let (access, signature) = parse_property_pre_fields(section)?;

    Some(PropertyDef {
        name,
        source_url,
        description,
        since,
        deprecated,
        signature: Some(signature),
        access: Some(access),
    })
}

fn parse_method_args(section: &ElementRef<'_>) -> (Vec<ArgumentDef>, Vec<ArgumentDef>) {
    let args = parse_variablelist_args(section);

    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    for arg in args {
        match arg.direction {
            Some(ArgDirection::In) => inputs.push(arg),
            Some(ArgDirection::Out) => outputs.push(arg),
            None => {}
        }
    }

    (inputs, outputs)
}

fn parse_signal_args(section: &ElementRef<'_>) -> Vec<ArgumentDef> {
    parse_variablelist_args(section)
}

fn parse_variablelist_args(section: &ElementRef<'_>) -> Vec<ArgumentDef> {
    let tr_sel = Selector::parse(".variablelist tbody tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();

    let mut args = Vec::new();

    for row in section.select(&tr_sel) {
        let mut cells = row.select(&td_sel);

        let Some(term_id) = cells.next() else {
            continue;
        };
        let Some(desc_td) = cells.next() else {
            continue;
        };

        let term_text = normalize_text(&term_id.text().collect::<String>());
        let desc_lines = collect_doc_lines_from_container(desc_td);

        let (direction, signature, name) = parse_variablelist_term(&term_text);

        args.push(ArgumentDef {
            name,
            signature,
            direction,
            description: desc_lines,
        });
    }

    args
}

fn parse_variablelist_term(
    text: &str,
) -> (Option<ArgDirection>, Option<String>, Option<String>) {
    // e.g.
    // "IN s identifier:"
    // "OUT ao devices:"
    // "s identifier"
    let normalized = text.trim().trim_end_matches(":");
    let parts: Vec<&str> = normalized.split_whitespace().collect();

    if parts.is_empty() {
        return (None, None, None);
    }

    match parts.as_slice() {
        [dir, sig, name, ..] if is_direction_token(dir) => (
            parse_direction_token(dir),
            Some((*sig).to_string()),
            Some((*name).to_string()),
        ),
        [sig, name, ..] => (
            None,
            Some((*sig).to_string()),
            Some((*name).to_string()),
        ),
        [single] => (None, None, Some((*single).to_string())),
        _ => (None, None, None),
    }
}

fn is_direction_token(token: &str) -> bool {
    matches!(token, "IN" | "OUT")
}

fn parse_direction_token(token: &str) -> Option<ArgDirection> {
    match token {
        "IN" => Some(ArgDirection::In),
        "OUT" => Some(ArgDirection::Out),
        _ => None,
    }
}

fn parse_property_pre_fields(section: &ElementRef<'_>) -> Option<(PropertyAccess, String)> {
    let pre = first_tag(section, "pre")?;
    let text = normalize_text(&pre.text().collect::<String>());

    if text.is_empty() {
        return None;
    }

    let parts: Vec<&str> = text.split_whitespace().collect();

    if parts.len() < 3 {
        return None;
    }

    let access = parse_property_access_token(parts[1])?;
    let signature = parts[2].to_string();

    Some((access, signature))
}

fn parse_property_access_token(token: &str) -> Option<PropertyAccess> {
    match token {
        "read" | "readonly" => Some(PropertyAccess::Read),
        "write" | "writeonly" => Some(PropertyAccess::Write),
        "readwrite" | "read/write" => Some(PropertyAccess::ReadWrite),
        _ => None,
    }
}

fn normalize_member_title(raw: &str, kind: &str) -> String {
    let text = raw.trim();

    // try match "The <name> <kind>"
    if let Some(inner) = text
        .strip_prefix("The ")
        .and_then(|s| s.strip_suffix(kind))
    {
        return inner
            .trim()
            .trim_end_matches("()")
            .trim()
            .trim_matches('"')
            .to_string()
    }

    // fallback
    text.to_string()
}
