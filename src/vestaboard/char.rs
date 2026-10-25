use std::fmt::{Display, Formatter};

use anyhow::{Result, anyhow};
use yansi::Paint;

use super::color::Color;

/// Vestaboard character.
///
/// https://docs.vestaboard.com/docs/charactercodes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Char(u8);

impl Char {
    pub const BLANK: Char = Char(0);

    /// Unset tile that the board shows blank.
    pub const EMPTY: Char = Char(u8::MAX);

    /// Convert a line of text to board characters.
    pub fn from_text(text: &str) -> Result<Vec<Char>> {
        text.chars().map(Char::try_from).collect()
    }

    /// Code sent to the board.
    pub fn code(self) -> u8 {
        if self == Char::EMPTY {
            return Char::BLANK.0;
        }

        self.0
    }
}

impl TryFrom<char> for Char {
    type Error = anyhow::Error;

    fn try_from(c: char) -> Result<Char> {
        let c = match c {
            '‘' | '’' | 'ʼ' | '`' => '\'',
            '“' | '”' => '"',
            '–' | '—' | '‐' => '-',
            '\t' | '\u{a0}' => ' ',
            'À'..='Å' | 'à'..='å' => 'A',
            'Ç' | 'ç' => 'C',
            'È'..='Ë' | 'è'..='ë' => 'E',
            'Ì'..='Ï' | 'ì'..='ï' => 'I',
            'Ñ' | 'ñ' => 'N',
            'Ò'..='Ö' | 'ò'..='ö' => 'O',
            'Ù'..='Ü' | 'ù'..='ü' => 'U',
            'Ý' | 'ý' | 'ÿ' => 'Y',
            c => c.to_ascii_uppercase(),
        };
        let code = match c {
            ' ' => 0,
            u @ 'A'..='Z' => u as u8 - b'A' + 1,
            d @ '1'..='9' => d as u8 - b'1' + 27,
            '0' => 36,
            p => PUNCTUATION
                .iter()
                .find(|(q, _)| *q == p)
                .map(|(_, code)| *code)
                .ok_or_else(|| anyhow!("the board can't show {c:?}"))?,
        };

        Ok(Char(code))
    }
}

impl From<Color> for Char {
    fn from(color: Color) -> Char {
        Char(color as u8)
    }
}

impl Display for Char {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if let Some(color) = Color::from_code(self.0) {
            return write!(f, "{}", "█".fg(color.into()));
        }

        let c = match self.code() {
            0 => ' ',
            code @ 1..=26 => (b'A' + code - 1) as char,
            code @ 27..=35 => (b'1' + code - 27) as char,
            36 => '0',
            code => PUNCTUATION
                .iter()
                .find(|(_, c)| *c == code)
                .map(|(p, _)| *p)
                .expect("Char only holds valid codes"),
        };

        write!(f, "{c}")
    }
}

const PUNCTUATION: [(char, u8); 20] = [
    ('!', 37),
    ('@', 38),
    ('#', 39),
    ('$', 40),
    ('(', 41),
    (')', 42),
    ('-', 44),
    ('+', 46),
    ('&', 47),
    ('=', 48),
    (';', 49),
    (':', 50),
    ('\'', 52),
    ('"', 53),
    ('%', 54),
    (',', 55),
    ('.', 56),
    ('/', 59),
    ('?', 60),
    ('°', 62),
];
