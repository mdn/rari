use crate::bq::NoteCard;

pub(crate) enum Flag {
    // TODO: fix this
    #[allow(dead_code)]
    Card(NoteCard),
    None,
}
