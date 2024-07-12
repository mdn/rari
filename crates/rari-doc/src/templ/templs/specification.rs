use rari_templ_func::rari_f;

use crate::error::DocError;

#[rari_f]
pub fn specification() -> Result<String, DocError> {
    let queries = env.browser_compat.join(",");
    let specs = env.spec_urls.join(",");
    Ok(format!(
        r#"<div class="bc-specs" data-bcd-query="{queries}" data-spec-urls="{specs}">
If you're able to see this, something went wrong on this page.
</div>"#
    ))
}
