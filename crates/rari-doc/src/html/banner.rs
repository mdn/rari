use rari_types::templ::TemplType;
use rari_types::{Arg, Quotes};
use tracing::{span, Level};

use crate::error::DocError;
use crate::pages::page::PageLike;
use crate::pages::types::utils::FmTempl;
use crate::templ::templs::invoke;

pub fn build_banner<T: PageLike>(banner: &FmTempl, page: &T) -> Result<String, DocError> {
    let rari_env = page.rari_env().ok_or(DocError::NoRariEnv)?;
    let (name, args) = match banner {
        FmTempl::NoArgs(name) => (name.as_str(), vec![]),
        FmTempl::WithArgs { name, args } => (
            name.as_str(),
            args.iter()
                .map(|s| Some(Arg::String(s.clone(), Quotes::Double)))
                .collect(),
        ),
    };
    let span = span!(Level::ERROR, "banner", banner = name,);
    let _enter = span.enter();
    let rendered_banner = match invoke(&rari_env, name, args) {
        Ok((rendered_banner, TemplType::Banner)) => rendered_banner,
        Ok((_, typ)) => {
            let span = span!(Level::ERROR, "banner", banner = name,);
            let _enter = span.enter();
            tracing::warn!("{typ} macro in banner frontmatter");
            Default::default()
        }
        Err(e) => {
            let span = span!(Level::ERROR, "banner", banner = name,);
            let _enter = span.enter();
            tracing::warn!("{e}");
            Default::default()
        }
    };
    Ok(rendered_banner)
}
