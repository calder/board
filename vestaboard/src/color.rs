/// Vestaboard color.
///
/// https://docs.vestaboard.com/docs/charactercodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Red = 63,
    Orange = 64,
    Yellow = 65,
    Green = 66,
    Blue = 67,
    Purple = 68,
    White = 69,
    Black = 70,
}

impl Color {
    pub fn from_code(code: u8) -> Option<Color> {
        Color::ALL.into_iter().find(|color| *color as u8 == code)
    }

    /// Emoji square typed for this color.
    pub fn square(self) -> char {
        match self {
            Color::Red => '🟥',
            Color::Orange => '🟧',
            Color::Yellow => '🟨',
            Color::Green => '🟩',
            Color::Blue => '🟦',
            Color::Purple => '🟪',
            Color::White => '⬜',
            Color::Black => '⬛',
        }
    }
}

#[cfg(feature = "terminal")]
impl From<Color> for yansi::Color {
    fn from(color: Color) -> yansi::Color {
        match color {
            Color::Red => yansi::Color::Red,
            Color::Orange => yansi::Color::Fixed(208),
            Color::Yellow => yansi::Color::Yellow,
            Color::Green => yansi::Color::Green,
            Color::Blue => yansi::Color::Blue,
            Color::Purple => yansi::Color::Magenta,
            Color::White => yansi::Color::White,
            Color::Black => yansi::Color::Black,
        }
    }
}

impl Color {
    pub const ALL: [Color; 8] = [
        Color::Red,
        Color::Orange,
        Color::Yellow,
        Color::Green,
        Color::Blue,
        Color::Purple,
        Color::White,
        Color::Black,
    ];
}
