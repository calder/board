use board::vestaboard::{Background, COLS, Char, Color, Grid, wrap};

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
