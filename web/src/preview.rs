use leptos::prelude::*;
use vestaboard::{COLS, Color, Grid, ROWS};

/// Draw a grid as board tiles with an optional note floating over it.
#[component]
pub fn Preview(
    grid: Memo<Grid>,
    invalid: Signal<bool>,
    note: Signal<Option<(String, bool)>>,
) -> impl IntoView {
    let tiles = (0..ROWS * COLS)
        .map(|i| {
            let tile = move || grid.with(|grid| grid[i / COLS][i % COLS]);
            let color = move || Color::from_code(tile().code());
            view! {
                <div
                    class="tile"
                    class:color=move || color().is_some()
                    style=move || color().map(|c| format!("--c:{}", css(c))).unwrap_or_default()
                >
                    {move || if color().is_some() { String::new() } else { tile().to_string() }}
                </div>
            }
        })
        .collect_view();
    // Keep the last words while the note fades out.
    let words = Memo::new(move |previous: Option<&String>| {
        note.with(|note| note.as_ref().map(|(words, _)| words.clone()))
            .or_else(|| previous.cloned())
            .unwrap_or_default()
    });

    view! {
        <div class="frame">
            <div
                class="board"
                class:invalid=move || invalid.get()
                role="img"
                aria-label="Board preview"
            >
                {tiles}
            </div>
            <p
                class="note"
                class:shown=move || note.with(Option::is_some)
                class:error=move || note.with(|note| note.as_ref().is_some_and(|(_, error)| *error))
                aria-live="polite"
            >
                {words}
            </p>
        </div>
    }
}

/// Pick the CSS color for a tile.
pub fn css(color: Color) -> &'static str {
    match color {
        Color::Red => "#d4312a",
        Color::Orange => "#f57c24",
        Color::Yellow => "#f7c623",
        Color::Green => "#2e9e4a",
        Color::Blue => "#1f7ad1",
        Color::Purple => "#7b3fa0",
        Color::White => "#e8e8e8",
        Color::Black => "#000000",
    }
}
