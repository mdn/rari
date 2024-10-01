use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f]
pub fn jsxref(
    api_name: String,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
) -> Result<String, DocError> {
    let display = display.as_deref().filter(|s| !s.is_empty());
    let global_objects = "Global_Objects";
    let display = display.unwrap_or(api_name.as_str());
    let mut url = format!("/{}/docs/Web/JavaScript/Reference/", &env.locale);
    let mut base_path = url.clone();

    let mut slug = api_name.replace("()", "").replace(".prototype.", ".");
    if !slug.contains("/") && slug.contains('.') {
        // Handle try...catch case
        slug = slug.replace('.', "/");
    }

    let page_url = format!("{url}{slug}");
    let object_page_url = format!("{url}{global_objects}/{slug}");

    let page = RariApi::get_page(&page_url);
    let object_page = RariApi::get_page(&object_page_url);
    if let Ok(_page) = page {
        url.push_str(&slug)
    } else if let Ok(_object_page) = object_page {
        base_path.extend([global_objects, "/"]);
        url.extend([global_objects, "/", &slug]);
    } else {
        url.push_str(&RariApi::decode_uri_component(&api_name));
    }

    if let Some(anchor) = anchor {
        if !anchor.starts_with('#') {
            url.push('#');
        }
        url.push_str(&anchor);
    }

    let code = !no_code.map(|nc| nc.as_bool()).unwrap_or_default();
    RariApi::link(&url, None, Some(display), code, None, false)
}
