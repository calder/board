use vestaboard::{Background, COLS, Char, Color, Grid, ROWS, wrap};

fn chars(text: &str) -> Vec<Char> {
    text.chars().map(|c| Char::try_from(c).unwrap()).collect()
}

#[test]
fn codes_round_trip() {
    for c in "ABCXYZ0123456789!@#$()-+&=;:'\"%,./?° ".chars() {
        assert_eq!(Char::try_from(c).unwrap().to_string(), c.to_string());
    }
    assert_eq!(Char::try_from('A').unwrap().code(), 1);
    assert_eq!(Char::try_from('a').unwrap().code(), 1);
    assert_eq!(Char::try_from('1').unwrap().code(), 27);
    assert_eq!(Char::try_from('0').unwrap().code(), 36);
}

#[test]
fn empty_shows_blank() {
    assert_eq!(Char::EMPTY.code(), Char::BLANK.code());
    assert_eq!(Char::EMPTY.to_string(), " ");
    assert_ne!(Char::EMPTY, Char::BLANK);
}

#[test]
fn colors_are_tiles() {
    let blue = Char::from(Color::Blue);
    assert_eq!(blue.code(), 67);
    assert_eq!(blue.to_string(), "\x1b[34m█\x1b[0m");
}

#[test]
fn round_trips_color_squares() {
    for color in Color::ALL {
        assert_eq!(Char::try_from(color.square()).unwrap(), Char::from(color));
        assert_eq!(Color::from_code(color as u8), Some(color));
    }
}

#[test]
fn reads_color_squares() {
    assert_eq!(
        Char::from_text("🟥A⬜\u{fe0f}").unwrap(),
        [
            Char::from(Color::Red),
            chars("A")[0],
            Char::from(Color::White)
        ]
    );
}

#[test]
fn normalizes_typography_and_accents() {
    assert_eq!(
        Char::from_text("Café — “it’s fine”").unwrap(),
        chars("CAFE - \"IT'S FINE\"")
    );
    assert_eq!(
        Char::from_text("JALAPEÑO Naïve Ångström").unwrap(),
        chars("JALAPENO NAIVE ANGSTROM")
    );
}

#[test]
fn rejects_emoji() {
    assert_eq!(
        Char::from_text("lol 😂").unwrap_err().to_string(),
        "the board can't show '😂'"
    );
}

#[test]
fn wraps_and_keeps_paragraphs() {
    assert_eq!(
        wrap(&[chars("AB CD EF")], 5, 3).unwrap(),
        [chars("AB CD"), chars("EF")]
    );
    assert_eq!(
        wrap(&[chars("AB"), chars("CD")], 22, 3).unwrap(),
        [chars("AB"), chars("CD")]
    );
    assert_eq!(wrap(&[chars("AB CD EF")], 2, 2), None);
    assert_eq!(wrap(&[chars("TOOLONGWORD")], 5, 3), None);
}

#[test]
fn keeps_spaces_and_blank_lines() {
    assert_eq!(
        wrap(&[chars("  A   B ")], 22, 6).unwrap(),
        [chars("  A   B ")]
    );
    assert_eq!(
        wrap(&[chars("A"), vec![], chars("B")], 22, 6).unwrap(),
        [chars("A"), vec![], chars("B")]
    );
    let grid = Grid::from_text("AB    CD              ").unwrap();
    assert_eq!(grid[2][..8], *chars("AB    CD"));
}

#[test]
fn centers_text() {
    let grid = Grid::from_text("hi").unwrap();
    let h = Char::try_from('H').unwrap();
    let i = Char::try_from('I').unwrap();
    assert_eq!(grid[2][10..13], [h, i, Char::EMPTY]);
}

#[test]
fn pads_and_fills_background() {
    let mut grid = Grid::from_text("hi\naya").unwrap();
    grid.pad();
    grid.fill(Background::Rainbow);
    assert_eq!(grid[0], [Char::from(Color::Red); COLS]);
    assert_eq!(grid[1][8..13], [Char::BLANK; 5]);
    assert_eq!(grid[1][7], Char::from(Color::Orange));
    assert_eq!(grid[2][8..10], [Char::BLANK; 2]);
    assert_eq!(grid[2][10], Char::try_from('H').unwrap());
    assert_eq!(grid[4][8..13], [Char::BLANK; 5]);
}

#[test]
fn explains_rejected_text() {
    assert_eq!(
        Grid::from_text("hi 😀").unwrap_err().to_string(),
        "the board can't show '😀'"
    );
    assert_eq!(
        Grid::from_text(&"word ".repeat(40))
            .unwrap_err()
            .to_string(),
        "text doesn't fit on the board"
    );
}

#[test]
fn reads_codes() {
    let mut codes = [[0; COLS]; ROWS];
    codes[0][..3].copy_from_slice(&[8, 9, 63]);
    let grid = Grid::from_codes(codes).unwrap();
    assert_eq!(grid[0][0], Char::try_from('H').unwrap());
    assert_eq!(grid[0][2], Char::from(Color::Red));
    assert_eq!(grid.codes(), codes);
}

#[test]
fn rejects_unknown_codes() {
    assert!(Char::try_from(43).is_err());
    let mut codes = [[0; COLS]; ROWS];
    codes[5][21] = 200;
    assert!(Grid::from_codes(codes).is_err());
}

#[test]
fn serializes_grid_as_codes() {
    let mut grid = Grid::from_text("HI").unwrap();
    grid.fill(Background::Blue);
    let json = serde_json::to_string(&grid).unwrap();
    assert!(json.starts_with("[[67,"));
    assert_eq!(serde_json::from_str::<Grid>(&json).unwrap(), grid);
}

#[test]
fn reads_code_rows() {
    let mut rows = vec![vec![0; COLS]; ROWS];
    rows[1][2] = 1;
    rows[5][21] = 63;
    let grid = Grid::from_code_rows(&rows).unwrap();
    assert_eq!(grid[1][2], Char::try_from('A').unwrap());
    assert_eq!(grid[5][21], Char::from(Color::Red));
    assert_eq!(grid[0][0], Char::BLANK);
}

#[test]
fn names_bad_code_rows() {
    let error = |rows: &[Vec<u32>]| Grid::from_code_rows(rows).unwrap_err().to_string();
    let rows = vec![vec![0; COLS]; ROWS];
    assert_eq!(error(&rows[..5]), "expected 6 rows, got 5");
    let mut short = rows.clone();
    short[3].pop();
    assert_eq!(error(&short), "row 3: expected 22 columns, got 21");
    let mut unknown = rows.clone();
    unknown[2][7] = 43;
    assert_eq!(
        error(&unknown),
        "row 2, column 7: unknown character code 43"
    );
    let mut huge = rows;
    huge[0][0] = 300;
    assert_eq!(error(&huge), "row 0, column 0: unknown character code 300");
}

#[test]
fn deserializing_names_bad_tiles() {
    let error = |json: &str| serde_json::from_str::<Grid>(json).unwrap_err().to_string();
    assert!(error("[[0]]").starts_with("expected 6 rows, got 1"));
    let mut rows = vec![vec![0; COLS]; ROWS];
    rows[2][7] = 43;
    let json = serde_json::to_string(&rows).unwrap();
    assert!(error(&json).starts_with("row 2, column 7: unknown character code 43"));
}
