use super::color::Color;
use super::grid::ROWS;

/// Fill for unset tiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
pub enum Background {
    Rainbow,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    White,
    Black,
}

impl Background {
    /// Pick the tile color for a row.
    pub fn color(self, row: usize) -> Color {
        match self {
            Background::Rainbow => RAINBOW[row],
            Background::Red => Color::Red,
            Background::Orange => Color::Orange,
            Background::Yellow => Color::Yellow,
            Background::Green => Color::Green,
            Background::Blue => Color::Blue,
            Background::Purple => Color::Purple,
            Background::White => Color::White,
            Background::Black => Color::Black,
        }
    }
}

const RAINBOW: [Color; ROWS] = [
    Color::Red,
    Color::Orange,
    Color::Yellow,
    Color::Green,
    Color::Blue,
    Color::Purple,
];

impl Background {
    pub const ALL: [Background; 9] = [
        Background::Rainbow,
        Background::Red,
        Background::Orange,
        Background::Yellow,
        Background::Green,
        Background::Blue,
        Background::Purple,
        Background::White,
        Background::Black,
    ];
}
