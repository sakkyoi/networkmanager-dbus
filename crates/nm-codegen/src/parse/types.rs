use anyhow::{Result, anyhow};
use scraper::{ElementRef, Html, Selector};

use crate::{
    parse::common::{
        collect_doc_lines_after_heading,
        collect_doc_lines_from_container,
        extract_since_and_deprecated,
        normalize_text,
    },
    model::types::{EnumDef, EnumValue, TypesPage},
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

        let description = collect_doc_lines_after_heading(&h3);
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
