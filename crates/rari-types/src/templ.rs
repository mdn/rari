use crate::{Arg, RariEnv};

pub type RariFn<R> = fn(&RariEnv<'_>, Vec<Option<Arg>>) -> R;
