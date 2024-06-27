use rari_templ_func::rari_f;

use crate::error::DocError;

#[rari_f]
pub fn compat() -> Result<String, DocError> {
    let multiple = env.browser_compat.len() > 1;
    Ok(env.browser_compat.iter().map(|query| format!(
        r#"<div class="bc-data" data-query="{query}" data-depth="1" data-multiple="{multiple}">
If you're able to see this, something went wrong on this page.
</div>"#)).collect::<Vec<String>>().join("\n"))
}

#[cfg(test)]
mod test {
    use rari_types::RariEnv;

    use crate::error::DocError;
    use crate::templ::render::{decode_ref, render};

    #[test]
    fn test_compat_none() -> Result<(), DocError> {
        let env = RariEnv {
            ..Default::default()
        };
        let (out, templs) = render(&env, r#"{{ compat }}"#)?;
        let out = decode_ref(&out, &templs)?;
        assert_eq!(out, r#""#);
        Ok(())
    }

    #[test]
    fn test_compat() -> Result<(), DocError> {
        let env = RariEnv {
            browser_compat: &["javascript.builtins.Array.concat".into()],
            ..Default::default()
        };
        let exp = r#"<div class="bc-data" data-query="javascript.builtins.Array.concat" data-depth="1" data-multiple="false">
If you're able to see this, something went wrong on this page.
</div>"#;
        let (out, templs) = render(&env, r#"{{ compat }}"#)?;
        let out = decode_ref(&out, &templs)?;
        assert_eq!(out, exp);
        Ok(())
    }

    #[test]
    fn test_compat_multiple() -> Result<(), DocError> {
        let env = RariEnv {
            browser_compat: &[
                "javascript.builtins.Array.concat".into(),
                "javascript.builtins.Array.filter".into(),
            ],
            ..Default::default()
        };
        let exp = r#"<div class="bc-data" data-query="javascript.builtins.Array.concat" data-depth="1" data-multiple="true">
If you're able to see this, something went wrong on this page.
</div>
<div class="bc-data" data-query="javascript.builtins.Array.filter" data-depth="1" data-multiple="true">
If you're able to see this, something went wrong on this page.
</div>"#;
        let (out, templs) = render(&env, r#"{{ compat }}"#)?;
        let out = decode_ref(&out, &templs)?;
        assert_eq!(out, exp);
        Ok(())
    }
}
