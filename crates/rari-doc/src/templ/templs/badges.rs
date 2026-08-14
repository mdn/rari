use rari_data::baseline::BaselineStatus;
use rari_templ_func::rari_f;
use rari_types::locale::Locale;

use crate::baseline::is_mocked_banner_section;
use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;

#[rari_f(register = "crate::Templ")]
pub fn experimental_inline() -> Result<String, DocError> {
    let mut out = String::new();
    write_experimental(&mut out, env.locale)?;
    Ok(out)
}

#[rari_f(register = "crate::Templ")]
pub fn experimentalbadge() -> Result<String, DocError> {
    experimental_inline(env)
}

#[rari_f(register = "crate::Templ")]
pub fn non_standard_inline() -> Result<String, DocError> {
    let mut out = String::new();
    write_non_standard(&mut out, env.locale)?;
    Ok(out)
}

#[rari_f(register = "crate::Templ")]
pub fn deprecated_inline() -> Result<String, DocError> {
    let mut out = String::new();
    if is_mocked_banner_section(env.slug) {
        write_baseline(&mut out, BaselineStatus::Discouraged, env.locale)?;
    } else {
        write_deprecated(&mut out, env.locale)?;
    }
    Ok(out)
}

#[rari_f(register = "crate::Templ")]
pub fn optional_inline() -> Result<String, DocError> {
    let str = l10n_json_data("Template", "optional", env.locale)?;
    Ok(format!(
        r#"<span class="badge inline optional">{str}</span>"#
    ))
}

pub fn write_experimental(out: &mut impl std::fmt::Write, locale: Locale) -> Result<(), DocError> {
    let title = l10n_json_data("Template", "experimental_badge_title", locale)?;
    let abbreviation = l10n_json_data("Template", "experimental_badge_abbreviation", locale)?;

    Ok(write_badge(out, title, abbreviation, "experimental")?)
}

pub fn write_non_standard(out: &mut impl std::fmt::Write, locale: Locale) -> Result<(), DocError> {
    let title = l10n_json_data("Template", "non_standard_badge_title", locale)?;
    let abbreviation = l10n_json_data("Template", "non_standard_badge_abbreviation", locale)?;

    Ok(write_badge(out, title, abbreviation, "nonstandard")?)
}

pub fn write_deprecated(out: &mut impl std::fmt::Write, locale: Locale) -> Result<(), DocError> {
    let title = l10n_json_data("Template", "deprecated_badge_title", locale)?;
    let abbreviation = l10n_json_data("Template", "deprecated_badge_abbreviation", locale)?;

    Ok(write_badge(out, title, abbreviation, "deprecated")?)
}

pub fn write_baseline(
    out: &mut impl std::fmt::Write,
    baseline: BaselineStatus,
    locale: Locale,
) -> Result<(), DocError> {
    let title = l10n_json_data("Template", &format!("baseline_{baseline}_title"), locale)?;
    let abbreviation = l10n_json_data(
        "Template",
        &format!("baseline_{baseline}_abbreviation"),
        locale,
    )?;

    Ok(write_badge(
        out,
        title,
        abbreviation,
        &format!("baseline {baseline}"),
    )?)
}

pub fn write_badge(
    out: &mut impl std::fmt::Write,
    title: &str,
    abbreviation: &str,
    typ: &str,
) -> std::fmt::Result {
    let title = html_escape::encode_quoted_attribute(title);
    write!(
        out,
        r#"<span role="img" class="icon icon-{typ}" title="{title}" aria-label="{abbreviation}"></span>"#
    )
}

#[cfg(test)]
mod tests {
    use rari_types::RariEnv;

    use super::*;

    fn deprecated_inline_for(slug: &str) -> String {
        let env = RariEnv {
            slug,
            locale: Locale::EnUs,
            ..Default::default()
        };
        deprecated_inline(&env).unwrap()
    }

    #[test]
    fn a_mocked_banner_section_gets_the_baseline_icon() {
        for slug in [
            "Web/API/AudioProcessingEvent",
            "WebAssembly/JavaScript_interface/Memory",
        ] {
            let out = deprecated_inline_for(slug);
            assert!(out.contains("icon-baseline discouraged"), "{slug}: {out}");
            assert!(!out.contains("icon-deprecated"), "{slug}: {out}");
        }
    }

    #[test]
    fn everywhere_else_keeps_the_deprecated_icon() {
        for slug in [
            "Web/Accessibility/ARIA/Reference/Attributes/aria-dropeffect",
            "Mozilla/Add-ons/WebExtensions/API/tabs",
            "Learn_web_development/Core/Scripting",
        ] {
            let out = deprecated_inline_for(slug);
            assert!(out.contains("icon-deprecated"), "{slug}: {out}");
            assert!(!out.contains("icon-baseline"), "{slug}: {out}");
        }
    }
}
