use colorful::{Color, Colorful};
use std::fmt::{self, Display};

const YELLOW: Color = Color::DarkGoldenrod; // 136

pub(crate) struct Ps2;

impl Display for Ps2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", "→ ".color(YELLOW))
    }
}
