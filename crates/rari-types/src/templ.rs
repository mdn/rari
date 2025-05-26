use crate::{Arg, RariEnv};

pub type RariFn<R> = fn(&RariEnv<'_>, Vec<Option<Arg>>) -> R;

pub trait RariF {
    type R;
    fn function() -> RariFn<Self::R>;
    fn doc() -> &'static str;
    fn outline() -> &'static str;
    fn is_sidebar() -> bool;
}
