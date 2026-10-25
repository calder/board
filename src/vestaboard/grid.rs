use std::fmt::{Display, Formatter};
use std::ops::Index;

use anyhow::{Result, bail};

use super::background::Background;
use super::char::Char;

pub const ROWS: usize = 6;
pub const COLS: usize = 22;

/// Full board text with unset tiles shown blank.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid([[Char; COLS]; ROWS]);

impl Grid {
    pub const EMPTY: Grid = Grid([[Char::EMPTY; COLS]; ROWS]);

    /// Centered text.
    pub fn from_text(text: &str) -> Result<Grid> {
        let paragraphs = text
            .split('\n')
            .map(Char::from_text)
            .collect::<Result<Vec<_>>>()?;
        let Some(lines) = wrap(&paragraphs, COLS, ROWS) else {
            bail!("text doesn't fit on the board")
        };
        let mut grid = Grid::EMPTY;
        grid.put_lines_centered(ROWS, &lines);

        Ok(grid)
    }

    /// Codes sent to the board.
    pub fn codes(&self) -> [[u8; COLS]; ROWS] {
        self.0.map(|row| row.map(Char::code))
    }

    /// Write a line.
    pub fn put(&mut self, row: usize, col: usize, line: &[Char]) {
        self.0[row][col..col + line.len()].copy_from_slice(line);
    }

    /// Write a centered line.
    pub fn put_centered(&mut self, row: usize, line: &[Char]) {
        self.put(row, (COLS - line.len()) / 2, line);
    }

    /// Write lines horizontally and vertically centered.
    pub fn put_lines_centered(&mut self, rows: usize, lines: &[Vec<Char>]) {
        let top = (rows - lines.len()) / 2;
        for (i, line) in lines.iter().enumerate() {
            self.put_centered(top + i, line);
        }
    }

    /// Blank unset tiles in a margin around the set ones.
    pub fn pad(&mut self) {
        let rows: Vec<usize> = (0..ROWS)
            .filter(|&r| self.0[r].iter().any(|&c| c != Char::EMPTY))
            .collect();
        let cols: Vec<usize> = (0..COLS)
            .filter(|&c| self.0.iter().any(|row| row[c] != Char::EMPTY))
            .collect();
        let (Some(&top), Some(&bottom), Some(&left), Some(&right)) =
            (rows.first(), rows.last(), cols.first(), cols.last())
        else {
            return;
        };
        for row in top.saturating_sub(1)..=(bottom + 1).min(ROWS - 1) {
            for col in left.saturating_sub(1)..=(right + 1).min(COLS - 1) {
                if self.0[row][col] == Char::EMPTY {
                    self.0[row][col] = Char::BLANK;
                }
            }
        }
    }

    /// Color unset tiles with a background.
    pub fn fill(&mut self, background: Background) {
        for (row, tiles) in self.0.iter_mut().enumerate() {
            for tile in tiles.iter_mut().filter(|tile| **tile == Char::EMPTY) {
                *tile = background.color(row).into();
            }
        }
    }
}

impl Index<usize> for Grid {
    type Output = [Char; COLS];

    fn index(&self, row: usize) -> &[Char; COLS] {
        &self.0[row]
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let line = "─".repeat(COLS);
        writeln!(f, "┌{line}┐")?;
        for row in &self.0 {
            write!(f, "│")?;
            for c in row {
                write!(f, "{c}")?;
            }
            writeln!(f, "│")?;
        }

        write!(f, "└{line}┘")
    }
}

/// Wrap text at word boundaries.
pub fn wrap(paragraphs: &[Vec<Char>], cols: usize, max_lines: usize) -> Option<Vec<Vec<Char>>> {
    let mut lines = Vec::new();
    for paragraph in paragraphs {
        let mut line = Vec::new();
        for word in paragraph
            .split(|c| *c == Char::BLANK)
            .filter(|w| !w.is_empty())
        {
            if word.len() > cols {
                return None;
            }
            if line.is_empty() {
                line.extend_from_slice(word);
            } else if line.len() + 1 + word.len() <= cols {
                line.push(Char::BLANK);
                line.extend_from_slice(word);
            } else {
                lines.push(std::mem::replace(&mut line, word.to_vec()));
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }

    (lines.len() <= max_lines).then_some(lines)
}
