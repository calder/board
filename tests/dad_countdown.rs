use std::time::Duration;

use board::State;
use jiff::Timestamp;
use vestaboard::Background;
use vestaboard::Grid;
use vestaboard::Message;

fn at(now: Timestamp, minutes: i64) -> Timestamp {
    if minutes >= 0 {
        now.checked_add(Duration::from_mins(minutes as u64))
            .unwrap()
    } else {
        now.checked_sub(Duration::from_mins((-minutes) as u64))
            .unwrap()
    }
}

fn text(eta_clock: &str, minutes: u32, background: Background) -> Grid {
    let unit = if minutes == 1 { "minute" } else { "minutes" };

    Message {
        text: format!("DAD IN TRANSIT\nETA {eta_clock}\n\nT-{minutes} {unit}"),
        pad: true,
        background: Some(background),
    }
    .to_grid()
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
    assert_eq!(state.displayed(now), text("19:42", 60, Background::Yellow));
    // T-30.
    assert_eq!(
        state.displayed(at(now, 12)),
        text("19:42", 30, Background::Yellow)
    );
    // T-10.
    assert_eq!(
        state.displayed(at(now, 32)),
        text("19:42", 10, Background::Yellow)
    );
    // Every minute from T-5.
    assert_eq!(
        state.displayed(at(now, 37)),
        text("19:42", 5, Background::Green)
    );
    assert_eq!(
        state.displayed(at(now, 38)),
        text("19:42", 4, Background::Green)
    );
    assert_eq!(
        state.displayed(at(now, 41)),
        text("19:42", 1, Background::Green)
    );
    // Hold T-1 past the ETA.
    assert_eq!(
        state.displayed(at(now, 45)),
        text("19:42", 1, Background::Green)
    );
    // Restore 10 minutes after arrival.
    assert_eq!(state.displayed(at(now, 52)), Grid::from_text(home).unwrap());
}

#[test]
fn pads_morning_hour() {
    let now = "2026-11-11T06:30:00-08:00".parse::<Timestamp>().unwrap();
    let eta = at(now, 35);
    let state = state("HOME", Some(eta));
    assert_eq!(state.displayed(now), text("07:05", 60, Background::Yellow));
}

#[test]
fn colors_dad_countdown_background() {
    let now = "2026-11-11T19:00:00-08:00".parse::<Timestamp>().unwrap();
    assert_eq!(
        state("HOME", Some(at(now, 61))).displayed(now),
        text("20:01", 90, Background::Red)
    );
    assert_eq!(
        state("HOME", Some(at(now, 60))).displayed(now),
        text("20:00", 60, Background::Yellow)
    );
    assert_eq!(
        state("HOME", Some(at(now, 6))).displayed(now),
        text("19:06", 10, Background::Yellow)
    );
    assert_eq!(
        state("HOME", Some(at(now, 5))).displayed(now),
        text("19:05", 5, Background::Green)
    );
    assert_eq!(
        state("HOME", Some(at(now, -1))).displayed(now),
        text("18:59", 1, Background::Green)
    );
}
