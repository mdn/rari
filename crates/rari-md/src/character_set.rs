// Copyright (c) 2017–2024, Asherah Connor and Comrak contributors
// This code is part of Comrak and is licensed under the BSD 2-Clause License.
// See LICENSE file for more information.
// Modified by Florian Dieminger in 2024
//
macro_rules! character_set {
    () => {{
        [false; 256]
    }};

    ($value:literal $(,$rest:literal)*) => {{
        const A: &[u8] = $value;
        let mut a = character_set!($($rest),*);
        let mut i = 0;
        while i < A.len() {
            a[A[i] as usize] = true;
            i += 1;
        }
        a
    }}
}

pub(crate) use character_set;
