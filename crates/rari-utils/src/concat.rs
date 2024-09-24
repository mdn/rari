#[macro_export]
macro_rules! concat_strs {
    ($($s:expr),+) => {{
        let mut len = 0;
        $(len += $s.len();)+
        let mut out = String::with_capacity(len);
        $(out.push_str($s);)+
        out
    }}
}
