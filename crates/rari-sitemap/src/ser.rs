use rari_types::globals::base_url;
use rari_utils::concat_strs;
use serde::Serializer;

pub(crate) fn prefix_base_url<S>(loc: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&concat_strs!(base_url(), loc))
}
