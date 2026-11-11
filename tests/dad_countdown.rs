use std::time::Duration;

use board::State;
use jiff::Timestamp;
use vestaboard::COLS;
use vestaboard::Char;
use vestaboard::Grid;

fn at(now: Timestamp, minutes: i64) -> Timestamp {
    if minutes >= 0 {
        now.checked_add(Duration::from_mins(minutes as u64))
            .unwrap()
    } else {
        now.checked_sub(Duration::from_mins((-minutes) as u64))
            .unwrap()
    }
}

fn text(eta_clock: &str, minutes: u32) -> Grid {
    let unit = if minutes == 1 { "minute" } else { "minutes" };

    Grid::from_text(&format!(
        "DAD IN TRANSIT\nETA {eta_clock}\n\nT-{minutes} {unit}"
    ))
    .unwrap()
}

fn state(grid: &str, eta: Option<Timestamp>) -> State {
    State {
        grid: Grid::from_text(grid).unwrap(),
        dad_countdown: eta,
    }
}

#[test]
fn shows_dad_countdown_then_restores() {
    let now = "2026-11-11T19:00:00-08:00".parse::<Timestamp>().unwrap();
    let home = "HOME";
    // 42 minutes out snaps to T-60.
    let eta = at(now, 42);
    let state = state(home, Some(eta));
    assert_eq!(state.displayed(now), text("19:42", 60));
    // T-30.
    assert_eq!(state.displayed(at(now, 12)), text("19:42", 30));
    // T-10.
    assert_eq!(state.displayed(at(now, 32)), text("19:42", 10));
    // Every minute from T-5.
    assert_eq!(state.displayed(at(now, 37)), text("19:42", 5));
    assert_eq!(state.displayed(at(now, 38)), text("19:42", 4));
    assert_eq!(state.displayed(at(now, 41)), text("19:42", 1));
    // Hold T-1 past the ETA.
    assert_eq!(state.displayed(at(now, 45)), text("19:42", 1));
    // Restore 10 minutes after arrival.
    assert_eq!(state.displayed(at(now, 52)), Grid::from_text(home).unwrap());
}

#[test]
fn pads_morning_hour() {
    let now = "2026-11-11T06:30:00-08:00".parse::<Timestamp>().unwrap();
    let eta = at(now, 35);
    let state = state("HOME", Some(eta));
    assert_eq!(state.displayed(now), text("07:05", 60));
}

#[test]
fn centers_dad_countdown() {
    let now = "2026-11-11T19:00:00-08:00".parse::<Timestamp>().unwrap();
    let grid = state("HOME", Some(at(now, 30))).displayed(now);
    // Four lines centered on six rows start at row 1.
    assert!(grid[0].iter().all(|tile| *tile == Char::EMPTY));
    assert!(grid[5].iter().all(|tile| *tile == Char::EMPTY));
    for (row, line) in [(1, "DAD IN TRANSIT"), (2, "ETA 19:30"), (4, "T-30 minutes")] {
        let chars = Char::from_text(line).unwrap();
        let col = (COLS - chars.len()) / 2;
        assert_eq!(grid[row][col..col + chars.len()], *chars, "{line}");
    }
    assert!(grid[3].iter().all(|tile| *tile == Char::EMPTY));
}
