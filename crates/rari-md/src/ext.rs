pub(crate) enum Flag {
    Card,
    None,
}

pub static DELIM_START: &str = "⟬";
pub static DELIM_START_LEN: usize = DELIM_START.len();
pub static DELIM_END: &str = "⟭";
pub static DELIM_END_LEN: usize = DELIM_END.len();
