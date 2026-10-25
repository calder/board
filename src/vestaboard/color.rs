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
        Some(match code {
            63 => Color::Red,
            64 => Color::Orange,
            65 => Color::Yellow,
            66 => Color::Green,
            67 => Color::Blue,
            68 => Color::Purple,
            69 => Color::White,
            70 => Color::Black,
            _ => return None,
        })
    }
}

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
