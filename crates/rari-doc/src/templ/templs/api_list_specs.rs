use std::collections::BTreeMap;

use itertools::Itertools;
use rari_templ_func::rari_f;
use tracing::warn;

use crate::error::DocError;
use crate::helpers::json_data::json_data_group;
use crate::helpers::subpages::write_li_with_badges;
use crate::pages::types::doc::Doc;
use crate::templ::index::{index_letter, index_letter_label_and_id, render_index_navigation};

/// Fragment ID prefix, shared by the navigation and the letter headings.
const INDEX_ID_PREFIX: &str = "index-specifications";

#[rari_f(register = "crate::Templ")]
pub fn listgroups() -> Result<String, DocError> {
    let group_data = json_data_group();

    let mut out_by_letter = BTreeMap::new();

    for (name, group) in group_data.iter().sorted_by(|(a, _), (b, _)| a.cmp(b)) {
        if let Some(overview) = group.overview.first() {
            let Some(first_letter) = overview.chars().next().map(index_letter) else {
                warn!("Skipping group {name} with an empty overview");
                continue;
            };
            let page = Doc::page_from_slug(
                &format!("Web/API/{}", overview.replace(' ', "_")),
                env.locale,
                true,
            )?;
            let out = out_by_letter.entry(first_letter).or_default();
            write_li_with_badges(out, &page, env.locale, false, true)?;
        }
    }

    let mut out = String::new();
    out.push_str(r#"<div class="index">"#);
    render_index_navigation(&mut out, out_by_letter.keys().copied(), INDEX_ID_PREFIX);
    for (letter, content) in out_by_letter {
        let (label, id) = index_letter_label_and_id(letter, INDEX_ID_PREFIX);
        out.extend([
            r#"<h3 id=""#,
            &id,
            r#"">"#,
            &label,
            r#"</h3><ul>"#,
            content.as_str(),
            r#"</ul>"#,
        ]);
    }
    out.push_str(r#"</div>"#);

    Ok(out)
}
