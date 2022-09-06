pub fn get_sentence_list<T: AsRef<str>>(elements: &[T]) -> String {
    let mut text = String::new();

    let len = elements.len();
    if len >= 2 {
        for (i, e) in elements.iter().enumerate() {
            use std::cmp::Ordering::*;
            text.push_str(e.as_ref());

            match i.cmp(&(len - 2)) {
                Equal => {
                    text.push_str(", and ");
                },
                Less => {
                    text.push_str(", ");
                },
                Greater => (),
            }
        }
    } else if len == 1 {
        text.push_str(elements[0].as_ref());
    }

    text
}

pub fn map_sentence_list<In, Out: AsRef<str>, M>(elements: &[In], mapper: M) -> String
where
    M: Fn(&In) -> Out,
{
    let mut text = String::new();

    let len = elements.len();
    if len >= 2 {
        for (i, e) in elements.iter().enumerate() {
            use std::cmp::Ordering::*;

            text.push_str(mapper(e).as_ref());

            match i.cmp(&(len - 2)) {
                Equal => {
                    text.push_str(", and ");
                },
                Less => {
                    text.push_str(", ");
                },
                Greater => (),
            }
        }
    } else if len == 1 {
        text.push_str(mapper(&elements[0]).as_ref());
    }

    text
}

use std::fmt;
pub fn ordinal_display(n: u8, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = n.to_string();

    let suffix = if s.ends_with('1') && !s.ends_with("11") {
        "st"
    } else if s.ends_with('2') && !s.ends_with("12") {
        "nd"
    } else if s.ends_with('3') && !s.ends_with("13") {
        "rd"
    } else {
        "th"
    };

    write!(f, "{}{}", s, suffix)
}
