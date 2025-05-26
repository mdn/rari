use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::helpers::web_ext_examples::web_ext_examples_json;
use crate::templ::api::RariApi;

#[rari_f(register = "crate::Templ")]
pub fn web_ext_all_examples() -> Result<String, DocError> {
    let mut out = String::new();

    let all_examples = web_ext_examples_json();

    out.extend([
        r#"<table class="standard-table fullwidth-table">"#,
        r#"<tr><th>Name</th><th>Description</th><th style="width: 40%">JavaScript APIs</th></tr>"#,
    ]);

    for example in all_examples {
        out.extend([
            r#"<tr><td><a href="https://github.com/mdn/webextensions-examples/tree/main/"#,
            &example.name,
            r#"">"#,
            &example.name,
            r#"</a></td><td>"#,
            &example.description,
            r#"</td><td>"#,
        ]);
        for api in &example.javascript_apis {
            let url = format!(
                "/{}/docs/Mozilla/Add-ons/WebExtensions/API/{}",
                env.locale.as_url_str(),
                &api.replace(' ', "_").replace("()", "").replace('.', "/"),
            );
            let link = RariApi::link(&url, env.locale, None, true, None, false)?;
            out.push_str(&link);
            out.push_str("<br/>")
        }
        out.push_str(r#"</tr>"#);
    }

    out.push_str(r#"</table>"#);

    Ok(out)
}
