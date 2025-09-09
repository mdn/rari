use std::borrow::Cow;
use std::collections::HashSet;

pub fn uniquify_id<'a>(ids: &mut HashSet<Cow<'_, str>>, id: Cow<'a, str>) -> Cow<'a, str> {
    if ids.contains(id.as_ref()) {
        let (prefix, mut count) = if let Some((prefix, counter)) = id.rsplit_once('_') {
            if counter.chars().all(|c| c.is_ascii_digit()) {
                let count = counter.parse::<i64>().unwrap_or_default() + 1;
                (prefix, count)
            } else {
                (id.as_ref(), 2)
            }
        } else {
            (id.as_ref(), 2)
        };
        let mut new_id = format!("{prefix}_{count}");
        while ids.contains(new_id.as_str()) && count < 666 {
            count += 1;
            new_id = format!("{prefix}_{count}");
        }
        ids.insert(Cow::Owned(new_id.clone()));
        return Cow::Owned(new_id);
    }
    let id_ = id.clone().to_string();
    ids.insert(Cow::Owned(id_));
    id
}
