pub(crate) type KeywordDocsMap = std::collections::HashMap<&'static str, &'static str>;

pub(crate) fn load_kw_docs() -> KeywordDocsMap {
    let mut map = KeywordDocsMap::new();
    map.insert("cssxref", "links to css");
    map
}
