use rari_templ_func::rari_f;
use rari_types::{AnyArg, ArgError};

use crate::error::DocError;
use crate::issues::get_issue_counter;
use crate::templ::api::RariApi;
use crate::templ::xref_index::resolve_xref;

/// Creates a link to a reference page resolved from the shared xref index.
#[rari_f(register = "crate::Templ")]
pub fn xref(
    name: String,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
) -> Result<String, DocError> {
    let (name, embedded_anchor) = split_name(&name);
    if name.is_empty() {
        return Err(DocError::ArgError(ArgError::at(
            "xref",
            1,
            "name",
            ArgError::MustNotBeEmpty,
        )));
    }

    let explicit_anchor = anchor.as_deref().filter(|anchor| !anchor.is_empty());
    if embedded_anchor.is_some() && explicit_anchor.is_some() {
        let ic = get_issue_counter();
        tracing::warn!(
            source = "templ-invalid-arg",
            ic = ic,
            arg = "anchor",
            "xref: `anchor` argument ignored because `name` already contains a fragment"
        );
    }
    let anchor = embedded_anchor.or(explicit_anchor);
    let slug = resolve_xref(name)?;
    let display = display.filter(|display| !display.is_empty());
    let display = display
        .as_deref()
        .unwrap_or_else(|| name.rsplit('/').next().unwrap_or(name));
    let mut url = format!("/{}/docs/{}", env.locale.as_url_str(), slug);
    if let Some(anchor) = anchor {
        if !anchor.starts_with('#') {
            url.push('#');
        }
        url.push_str(anchor);
    }

    RariApi::link(
        &url,
        Some(env.locale),
        Some(display),
        !no_code.map(|arg| arg.as_bool()).unwrap_or_default(),
        None,
        false,
    )
}

fn split_name(name: &str) -> (&str, Option<&str>) {
    match name.split_once('#') {
        Some((name, anchor)) => (name, Some(anchor)),
        None => (name, None),
    }
}

#[cfg(test)]
mod tests {
    use super::split_name;

    #[test]
    fn fragments_are_split_from_names() {
        let cases = [
            ("alert_role", ("alert_role", None)),
            ("aria-sort#examples", ("aria-sort", Some("examples"))),
            (
                "ARIA/alert_role#examples",
                ("ARIA/alert_role", Some("examples")),
            ),
        ];
        for (name, expected) in cases {
            assert_eq!(split_name(name), expected, "[{name}]");
        }
    }
}
