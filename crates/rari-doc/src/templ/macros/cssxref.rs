use rari_templ_func::rari_f;
use rari_types::fm_types::PageType;

use crate::docs::page::PageLike;
use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f]
pub fn cssxref(
    name: String,
    display_name: Option<String>,
    anchor: Option<String>,
) -> Result<String, DocError> {
    let maybe_display_name = display_name
        .as_deref()
        .or_else(|| name.rsplit_once('/').map(|(_, s)| s))
        .unwrap_or(name.as_str());
    let mut slug = name
        .strip_prefix("&lt;")
        .unwrap_or(name.strip_prefix('<').unwrap_or(name.as_str()));
    slug = slug
        .strip_suffix("&gt;")
        .unwrap_or(slug.strip_suffix('>').unwrap_or(slug));
    slug = slug.strip_suffix("()").unwrap_or(slug);

    let slug = match name.as_str() {
        "&lt;color&gt;" => "color_value",
        "&lt;flex&gt;" => "flex_value",
        "&lt;overflow&gt;" => "overflow_value",
        "&lt;position&gt;" => "position_value",
        ":host()" => ":host_function",
        "fit-content()" => "fit_content_function",
        _ => slug,
    };

    let url = format!(
        "/{}/docs/Web/CSS/{slug}{}",
        env.locale.as_url_str(),
        anchor.as_deref().unwrap_or_default()
    );

    let display_name = if display_name.is_some() {
        maybe_display_name.to_string()
    } else if let Ok(doc) = RariApi::get_page(&url) {
        match doc.page_type() {
            PageType::CssFunction if !maybe_display_name.ends_with("()") => {
                format!("{maybe_display_name}()")
            }
            PageType::CssType
                if !(maybe_display_name.starts_with("&lt;")
                    && maybe_display_name.ends_with("&gt;")) =>
            {
                format!("&lt;{maybe_display_name}&gt;")
            }
            _ => maybe_display_name.to_string(),
        }
    } else {
        maybe_display_name.to_string()
    };

    Ok(format!(r#"<a href={url}><code>{display_name}</code></a>"#))
}
/*
<%
/*
  Inserts a link to a CSS entity documentation
  Appropriate styling is applied.

  This template handles CSS data types and CSS functions gracefully by
  automatically adding arrow brackets or round brackets, respectively.

  For the ease of linking to CSS descriptors and functions, if only one
  parameter is specified and it contains a slash, the displayed link name
  will strip the last slash and any content before it.

  Parameters:
  $0 - API name to refer to
  $1 - name of the link to display (optional)
  $2 - anchor within the article to jump to of the form '#xyz' (optional)

  Examples:
  {{cssxref("background")}} =>
      <a href="/en-US/docs/Web/CSS/background"><code>background</code></a>
  {{cssxref("length")}} =>
      <a href="/en-US/docs/Web/CSS/length"><code>&lt;length&gt;</code></a>
  {{cssxref("gradient/linear-gradient")}} =>
      <a href="/en-US/docs/Web/CSS/gradient/linear-gradient"><code>linear-gradient()</code></a>
  {{cssxref("calc()")}} =>
      <a href="/en-US/docs/Web/CSS/calc"><code>calc()</code></a>
  {{cssxref("margin-top", "top margin")}} =>
      <a href="/en-US/docs/Web/CSS/margin-top"><code>top margin</code></a>
  {{cssxref("attr()", "", "#values")}} =>
      <a href="/en-US/docs/Web/CSS/attr#values"><code>attr()</code></a>
*/

const lang = env.locale;
let url = "";
let urlWithoutAnchor = "";
let displayName = ($1 || $0.slice($0.lastIndexOf("/") + 1));

// Deal with CSS data types and functions by removing <> and ()
let slug = $0.replace(/&lt;(.*)&gt;/g, "$1")
    .replace(/\(\)/g, "");

// Special case <color>, <flex>, <overflow>, and <position>
if (/^&lt;(color|flex|overflow|position)&gt;$/.test($0)) {
    slug += "_value";
}

// Special case :host() and fit-content()
if (/^(:host|fit-content)\(\)$/.test($0)) {
    slug += "_function";
}

const basePath = `/${lang}/docs/Web/CSS/`;
urlWithoutAnchor = basePath + slug;
url = urlWithoutAnchor + $2;

const thisPage = (!$1 || !$2) ?
  wiki.getPage(`/en-US/docs/Web/CSS/${slug}`) :
  null;

if (!$1) {
    // Append parameter brackets to CSS functions
    if ((thisPage.pageType === "css-function") && !displayName.endsWith("()")) {
        displayName += "()";
    }
    // Enclose CSS data types in arrow brackets
    if ((thisPage.pageType === "css-type") && !/^&lt;.+&gt;$/.test(displayName)) {
        displayName = "&lt;" + displayName + "&gt;";
    }
}

const entry = web.smartLink(url, "", `<code>${displayName}</code>`, $0, basePath);

%><%- entry %>
*/
