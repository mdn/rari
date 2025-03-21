use rari_doc::templ::templs::{embeds::embed_live_sample, links::cssxref};

pub(crate) type KeywordDocsMap = std::collections::HashMap<&'static str, &'static str>;

pub(crate) fn load_kw_docs() -> KeywordDocsMap {
    let mut map = KeywordDocsMap::new();
    map.insert("cssxref", cssxref::OUTLINE_FOR_CSSXREF);
    map.insert(
        "embedlivesample",
        embed_live_sample::OUTLINE_FOR_EMBED_LIVE_SAMPLE,
    );
    map
}
