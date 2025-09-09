use rari_doc::pages::page::{Page, PageLike};
use rari_types::locale::Locale;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::issues::fix_page;
use crate::error::ToolError;

pub fn fix_all(docs: &[Page], locale: Option<Locale>) -> Result<Vec<&Page>, ToolError> {
    docs.into_par_iter()
        .filter(|page| locale.map(|locale| page.locale() == locale).unwrap_or(true))
        .map(|page| (fix_page(page), page))
        .filter_map(|(res, page)| {
            if matches!(res, Ok(false)) {
                None
            } else {
                Some(res.map(|_| page))
            }
        })
        .collect()
}
