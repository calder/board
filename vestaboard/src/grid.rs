use std::fmt::{Display, Formatter};
use std::ops::Index;

use anyhow::{Result, anyhow, bail, ensure};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

    /// Grid of codes read from the board.
    pub fn from_codes(codes: [[u8; COLS]; ROWS]) -> Result<Grid> {
        let mut grid = Grid::EMPTY;
        for (row, codes) in codes.iter().enumerate() {
            for (col, &code) in codes.iter().enumerate() {
                grid.0[row][col] = Char::try_from(code)?;
            }
        }

        Ok(grid)
    }

    /// Read rows of codes and name the first bad row or column counting from 0.
    pub fn from_code_rows(rows: &[Vec<u32>]) -> Result<Grid> {
        ensure!(
            rows.len() == ROWS,
            "expected {ROWS} rows, got {}",
            rows.len()
        );
        let mut grid = Grid::EMPTY;
        for (row, codes) in rows.iter().enumerate() {
            ensure!(
                codes.len() == COLS,
                "row {row}: expected {COLS} columns, got {}",
                codes.len()
            );
            for (col, &code) in codes.iter().enumerate() {
                grid.0[row][col] = u8::try_from(code)
                    .ok()
                    .and_then(|code| Char::try_from(code).ok())
                    .ok_or_else(|| {
                        anyhow!("row {row}, column {col}: unknown character code {code}")
                    })?;
            }
        }

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

/// Serialized as the codes the board shows so unset tiles become blank.
impl Serialize for Grid {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.codes().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Grid {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Grid, D::Error> {
        let codes = <[[u8; COLS]; ROWS]>::deserialize(deserializer)?;

        Grid::from_codes(codes).map_err(serde::de::Error::custom)
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

/// Wrap lines longer than `cols` at their last fitting space.
pub fn wrap(paragraphs: &[Vec<Char>], cols: usize, max_lines: usize) -> Option<Vec<Vec<Char>>> {
    // Spaces are kept as typed so people can align text by hand.
    let mut lines = Vec::new();
    for paragraph in paragraphs {
        let mut rest = paragraph.as_slice();
        while rest.len() > cols {
            let split = rest[..=cols].iter().rposition(|c| *c == Char::BLANK)?;
            lines.push(rest[..split].to_vec());
            rest = &rest[split + 1..];
        }
        lines.push(rest.to_vec());
    }

    (lines.len() <= max_lines).then_some(lines)
}
