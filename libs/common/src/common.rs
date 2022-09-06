#![allow(non_snake_case)]

#[macro_export]
macro_rules! d {
    () => {
        Default::default()
    };
}

#[macro_export]
macro_rules! nu8 {
    ($byte:expr) => {{
        use std::num::NonZeroU8;
        NonZeroU8::new($byte).unwrap()
    }};
}

#[cfg(test)]
extern crate quickcheck;

extern crate platform_types;

extern crate inner_common;
pub use inner_common::*;

extern crate features;
pub use features::*;

mod rendering;
pub use self::rendering::*;

mod text;
pub use self::text::*;

mod gui;
pub use self::gui::*;

mod traits;
pub use self::traits::*;

