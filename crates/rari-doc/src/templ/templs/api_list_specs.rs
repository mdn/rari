use std::collections::BTreeMap;

use itertools::Itertools;
use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::helpers::json_data::json_data_group;
use crate::helpers::subpages::write_li_with_badges;
use crate::pages::types::doc::Doc;

#[rari_f(crate::Templ)]
pub fn api_list_specs() -> Result<String, DocError> {
    let group_data = json_data_group();

    let mut out_by_letter = BTreeMap::new();

    for (name, group) in group_data.iter().sorted_by(|(a, _), (b, _)| a.cmp(b)) {
        if let Some(overview) = group.overview.first() {
            let first_letter = name.chars().next().unwrap_or_default();
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
    for (letter, content) in out_by_letter {
        out.push_str(r#"<h3>"#);
        out.push(letter);
        out.extend([r#"</h3><ul>"#, content.as_str(), r#"</ul>"#]);
    }
    out.push_str(r#"</div>"#);

    Ok(out)
}
