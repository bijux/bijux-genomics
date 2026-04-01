use std::fmt::Display;

use super::{HOW, MORE, WHY};

pub fn message(what: impl Display) -> String {
    format!("WHAT: {what}\nWHY: {WHY}\nHOW: {HOW}\nMORE: {MORE}")
}
