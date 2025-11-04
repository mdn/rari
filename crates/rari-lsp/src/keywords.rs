use rari_doc::templ::templs::TEMPL_MAP;
use rari_doc::Templ;

pub(crate) type KeywordDocsMap = std::collections::HashMap<&'static str, &'static Templ>;

pub(crate) fn load_kw_docs() -> KeywordDocsMap {
    TEMPL_MAP.iter().map(|t| (t.name, *t)).collect()
}
