use rari_templ_func::rari_f;
use rari_types::fm_types::PageType;
use rari_utils::concat_strs;

use crate::error::DocError;
use crate::helpers::subpages::{get_sub_pages, SubPagesSorter};
use crate::pages::page::PageLike;
use crate::templ::api::RariApi;

#[rari_f(register = "crate::Templ")]
pub fn apilistalpha() -> Result<String, DocError> {
    let mut out = String::new();
    let pages = get_sub_pages("/en-US/docs/Web/API", Some(1), SubPagesSorter::Title)?;

    let mut current_letter = None;

    out.push_str(r#"<div class="index">"#);
    for page in pages
        .iter()
        .filter(|page| page.page_type() == PageType::WebApiInterface)
    {
        let first_letter = page.title().chars().next();

        if first_letter != current_letter {
            if current_letter.is_some() {
                out.push_str("</ul>");
            }
            current_letter = first_letter;
            if let Some(current_letter) = current_letter {
                out.push_str("<h3>");
                out.push_str(&html_escape::encode_safe(
                    current_letter.encode_utf8(&mut [0; 4]),
                ));
                out.push_str("</h3><ul>");
            }
        }

        let url = concat_strs!("/", env.locale.as_url_str(), "/docs/", page.slug());

        out.extend([
            "<li>",
            &RariApi::link(
                &url,
                env.locale,
                None,
                true,
                Some(page.short_title().unwrap_or(page.title())),
                true,
            )?,
            "</li>",
        ]);
    }
    out.push_str(r#"</div>"#);

    Ok(out)
}
