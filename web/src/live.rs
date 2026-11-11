use leptos::prelude::*;
use vestaboard::Grid;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{EventSource, MessageEvent};

/// Keep `board` updated from the board server and `connected` true while its stream is up.
pub fn follow(board: RwSignal<Option<Grid>>, connected: RwSignal<bool>) {
    let Ok(events) = EventSource::new("/events") else {
        return;
    };
    let on_message = Closure::<dyn Fn(MessageEvent)>::new(move |event: MessageEvent| {
        if let Some(data) = event.data().as_string()
            && let Ok(grid) = serde_json::from_str::<Grid>(&data)
        {
            board.set(Some(grid));
            connected.set(true);
        }
    });
    // The browser reconnects by itself after an error.
    let on_error = Closure::<dyn Fn()>::new(move || connected.set(false));
    events.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    events.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    // The stream lasts as long as the page.
    on_message.forget();
    on_error.forget();
}
