use anyhow::Result;
use scraper::{ElementRef, Html, Selector};

use crate::{
    model::types::{EnumDef, EnumValue, TypesPage},
    parse::common::{
        collect_doc_lines_from_container, direct_children_with_class,
        extract_since_and_deprecated, find_refentry, first_heading_text, normalize_text,
        parse_refentry_page_identity, parse_section_detail,
    },
};

pub fn parse_types_page(html: &str, base_url: &str) -> Result<TypesPage> {
    let doc = Html::parse_document(html);
    let Some(refentry) = find_refentry(&doc) else {
        return Ok(TypesPage {
            source_url: Some(base_url.to_string()),
            ..Default::default()
        });
    };

    let (_, page_description) = parse_refentry_page_identity(&refentry);

    let mut enums = Vec::new();

    for section in direct_children_with_class(&refentry, "refsect2") {
        if let Some(enum_def) = parse_enum_detail(&section, base_url) {
            enums.push(enum_def);
        }
    }

    Ok(TypesPage {
        description: page_description,
        source_url: Some(base_url.to_string()),
        enums,
    })
}

fn parse_enum_detail(section: &ElementRef<'_>, base_url: &str) -> Option<EnumDef> {
    let detail = parse_section_detail(section, "h3", base_url)?;
    let name = detail.title.strip_prefix("enum ")?.to_string();
    let values_section = find_values_section(section)?;
    let values = parse_enum_values(&values_section);

    if values.is_empty() {
        return None;
    }

    Some(EnumDef {
        name,
        description: detail.description,
        values,
        source_url: detail.source_url,
        since: detail.since,
        deprecated: detail.deprecated,
    })
}

fn find_values_section<'a>(section: &'a ElementRef<'a>) -> Option<ElementRef<'a>> {
    for sub in direct_children_with_class(section, "refsect3") {
        let Some(title) = first_heading_text(&sub) else {
            continue;
        };

        if title == "Values" {
            return Some(sub);
        }
    }

    None
}

fn parse_enum_values(section: &ElementRef<'_>) -> Vec<EnumValue> {
    let row_sel = Selector::parse("tbody tr").unwrap();
    let name_sel = Selector::parse(".enum_member_name p").unwrap();
    let value_sel = Selector::parse(".enum_member_value code").unwrap();
    let desc_cell_sel = Selector::parse(".enum_member_description").unwrap();

    let mut values = Vec::new();

    for row in section.select(&row_sel) {
        let name = row
            .select(&name_sel)
            .next()
            .map(|n| normalize_text(&n.text().collect::<String>()))
            .unwrap_or_default();

        if name.is_empty() {
            continue;
        }

        let Some(value) = row
            .select(&value_sel)
            .next()
            .map(|v| normalize_text(&v.text().collect::<String>()))
        else {
            continue;
        };

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

    values
}
