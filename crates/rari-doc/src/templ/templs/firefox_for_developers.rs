use std::fmt::Display;

use rari_templ_func::rari_f;
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;
use crate::html::links::{render_link_via_page, LinkFlags};

const OLD_VERSIONS: &[&str] = &["3.6", "3.5", "3", "2", "1.5"];

#[rari_f]
pub fn firefox_for_developers() -> Result<String, DocError> {
    let locale = env.locale;
    let slug = env.slug;

    let version_str = slug
        .split('/')
        .next_back()
        .ok_or_else(|| invalid_slug(slug))?;

    // Determine if version_str is a float (for OLD_VERSIONS) or integer
    let mut max_version: i32 = if let Ok(int_version) = version_str.parse::<i32>() {
        int_version
    } else if OLD_VERSIONS.contains(&version_str) {
        // If version_str is in OLD_VERSIONS, treat it as 3 (since all are < 4)
        3
    } else {
        return Err(invalid_slug(slug));
    };

    let mut old_version_start_idx = 0;

    // Determine the start index of old version and the max version
    if max_version < 4 {
        old_version_start_idx = OLD_VERSIONS.iter().position(|&v| v == version_str).unwrap() + 1;
    } else {
        max_version -= 1;
    }

    let mut min_version = max_version - 30;

    if min_version < 4 {
        min_version = 4;
    }

    let mut out = String::new();
    out.push_str(r#"<div class="multiColumnList"><ul>"#);

    // For newer version
    for version in (min_version..=max_version).rev() {
        generate_release_link(&mut out, version, locale)?;
    }

    // For older version
    if min_version == 4 {
        for version in OLD_VERSIONS.iter().skip(old_version_start_idx) {
            generate_release_link(&mut out, version, locale)?;
        }
    }

    out.push_str("</ul></div>");
    Ok(out)
}

fn invalid_slug(slug: &str) -> DocError {
    DocError::InvalidSlugForX(format!("{slug}: firefox_for_developers templ"))
}

fn generate_release_link<T: Display>(
    out: &mut String,
    version: T,
    locale: Locale,
) -> Result<(), DocError> {
    let for_developers = l10n_json_data("Template", "for_developers", locale)?;
    out.push_str("<li>");
    render_link_via_page(
        out,
        &format!("/Mozilla/Firefox/Releases/{version}"),
        locale,
        Some(&format!("Firefox {version} {for_developers}")),
        None,
        LinkFlags {
            code: false,
            with_badges: false,
            report: false,
        },
    )?;
    out.push_str("</li>");
    Ok(())
}
