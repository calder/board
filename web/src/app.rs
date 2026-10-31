use std::iter::once;
use std::time::Duration;

use gloo_net::http::Request;
use leptos::html::Textarea;
use leptos::prelude::*;
use leptos::task::spawn_local;
use vestaboard::{Background, Color, Grid, Message};
use web_sys::{HtmlElement, UrlSearchParams};

use crate::preview::{Preview, css};

#[derive(Clone, PartialEq)]
enum Status {
    Idle,
    Sending,
    Sent,
    Failed(String),
}

/// Compose a message with a live preview and send it to the board.
#[component]
pub fn App() -> impl IntoView {
    let text = RwSignal::new(param("text").unwrap_or_default());
    let pad = RwSignal::new(param("pad").or_else(|| stored("pad")).as_deref() != Some("0"));
    let background = RwSignal::new(
        param("background")
            .or_else(|| stored("background"))
            .and_then(|name| backgrounds().find(|b| format!("{b:?}").eq_ignore_ascii_case(&name))),
    );
    Effect::watch(
        move || pad.get(),
        |&on, _, _| store("pad", if on { "1" } else { "0" }),
        false,
    );
    Effect::watch(
        move || background.get(),
        |&background, _, _| {
            let name = background.map_or("none".to_string(), |b| format!("{b:?}").to_lowercase());
            store("background", &name);
        },
        false,
    );
    let picking = RwSignal::new(false);
    let status = RwSignal::new(Status::Idle);
    let textarea = NodeRef::<Textarea>::new();

    let message = Signal::derive(move || Message {
        text: text.get(),
        pad: pad.get(),
        background: background.get(),
    });
    let layout = Memo::new(move |_| message.with(|m| m.to_grid().map_err(|err| err.to_string())));
    // Keep showing the last good layout while the text can't be shown.
    let grid = Memo::new(move |previous: Option<&Grid>| match layout.get() {
        Ok(grid) => grid,
        Err(_) => previous.copied().unwrap_or(Grid::EMPTY),
    });
    let invalid = Signal::derive(move || layout.with(Result::is_err));
    let can_send = Signal::derive(move || !invalid.get() && status.get() != Status::Sending);
    // An empty message with no background blanks the board.
    let clearing =
        Signal::derive(move || text.with(String::is_empty) && background.get().is_none());
    Effect::new(move |_| {
        message.track();
        status.set(Status::Idle);
    });
    // Focus right away with a mouse but leave phones' keyboards closed until tapped.
    Effect::new(move |_| {
        if let Some(area) = textarea.get()
            && has_mouse()
        {
            let _ = area.focus();
        }
    });

    let submit = move || {
        if !can_send.get_untracked() {
            return;
        }
        let message = message.get_untracked();
        status.set(Status::Sending);
        spawn_local(async move {
            status.set(match send(&message).await {
                Ok(()) => Status::Sent,
                Err(err) => Status::Failed(err),
            });
            set_timeout(
                move || {
                    if status.get_untracked() == Status::Sent {
                        status.set(Status::Idle);
                    }
                },
                Duration::from_secs(2),
            );
        });
    };
    // Grow the text box to fit everything typed so nothing hides past its edge.
    let resized = RwSignal::new(0);
    let _ = window_event_listener(leptos::ev::resize, move |_| resized.update(|n| *n += 1));
    Effect::new(move |_| {
        resized.track();
        let current = text.get();
        if let Some(area) = textarea.get() {
            // Measure the latest text even if Leptos hasn't copied it into the box yet.
            if area.value() != current {
                area.set_value(&current);
            }
            let style = HtmlElement::style(&area);
            let _ = style.set_property("height", "auto");
            let _ = style.set_property("height", &format!("{}px", area.scroll_height()));
        }
    });
    let insert = move |color: Color| {
        let Some(area) = textarea.get() else {
            return;
        };
        let start = area.selection_start().ok().flatten().unwrap_or(0);
        let end = area.selection_end().ok().flatten().unwrap_or(start);
        let _ = area.set_range_text_with_start_and_end_and_mode(
            &color.square().to_string(),
            start,
            end,
            "end",
        );
        text.set(area.value());
        let _ = area.focus();
    };
    let note = Signal::derive(move || match (layout.get(), status.get()) {
        (Err(err), _) | (_, Status::Failed(err)) => Some((sentence(&err), true)),
        (_, Status::Sending) => Some(("Sending...".into(), false)),
        (_, Status::Sent) if clearing.get() => Some(("Cleared".into(), false)),
        (_, Status::Sent) => Some(("Sent".into(), false)),
        (_, Status::Idle) => None,
    });
    let focus = move |_| {
        if let Some(area) = textarea.get() {
            let _ = area.focus();
        }
    };

    view! {
        <main class="app">
            <div class="stage" on:click=focus>
                <Preview grid=grid invalid=invalid note=note />
            </div>
            <div class="compose" on:click=focus>
                <textarea
                    node_ref=textarea
                    rows="1"
                    placeholder="Type a message"
                    autocapitalize="characters"
                    autocomplete="off"
                    spellcheck="false"
                    aria-label="Message"
                    prop:value=move || text.get()
                    on:input=move |ev| text.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" && (ev.meta_key() || ev.ctrl_key()) {
                            ev.prevent_default();
                            submit();
                        }
                    }
                ></textarea>
            </div>
            <div class="toolbar">
                <div class="picker">
                    <button
                        class="round"
                        title="Background"
                        aria-label="Background"
                        aria-expanded=move || picking.get().to_string()
                        on:pointerdown=|ev| ev.prevent_default()
                        on:click=move |_| picking.update(|open| *open = !*open)
                    >
                        <span class="fill">
                            <svg width="20" height="18" viewBox="0 0 20 18" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linejoin="round" stroke-linecap="round">
                                <path d="M2.5 8 8.5 2l6 6-6 6z" />
                                <path d="M5 5.5 3 3.5" />
                                <path d="M17.6 11.4c0 .9-.6 1.6-1.4 1.6s-1.4-.7-1.4-1.6c0-.9 1.4-2.6 1.4-2.6s1.4 1.7 1.4 2.6z" fill="currentColor" stroke="none" />
                            </svg>
                            {move || fill_bar(background.get())}
                        </span>
                    </button>
                    <Show when=move || picking.get()>
                        <div
                            class="backdrop"
                            on:pointerdown=|ev| ev.prevent_default()
                            on:click=move |_| picking.set(false)
                        ></div>
                        <div class="options">
                            <div class="choices" role="radiogroup" aria-label="Background">
                            {once(None)
                                .chain(backgrounds().map(Some))
                                .map(|option| {
                                    let label = option.map_or("No background".to_string(), |b| format!("{b:?}"));
                                    view! {
                                        <button
                                            class="option"
                                            role="radio"
                                            title=label.clone()
                                            aria-label=label
                                            aria-checked=move || (background.get() == option).to_string()
                                            on:pointerdown=|ev| ev.prevent_default()
                                            on:click=move |_| {
                                                background.set(option);
                                                picking.set(false);
                                            }
                                        >
                                            {mini_board(option)}
                                        </button>
                                    }
                                })
                                .collect_view()}
                            </div>
                            <hr />
                            <button
                                class="option margin"
                                title="Margin around text"
                                aria-label="Margin around text"
                                aria-pressed=move || pad.get().to_string()
                                on:pointerdown=|ev| ev.prevent_default()
                                on:click=move |_| pad.update(|on| *on = !*on)
                            >
                                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M3 8V4h4M17 4h4v4M21 16v4h-4M7 20H3v-4" />
                                    <rect x="8" y="10.5" width="8" height="3" rx="1" fill="currentColor" stroke="none" />
                                </svg>
                            </button>
                        </div>
                    </Show>
                </div>
                <div class="inserts" aria-label="Insert a color tile">
                    {colors()
                        .map(|color| {
                            let label = format!("Insert {color:?} tile");
                            view! {
                                <button
                                    class="insert"
                                    style=format!("--c:{}", css(color))
                                    title=label.clone()
                                    aria-label=label
                                    on:pointerdown=|ev| ev.prevent_default()
                                    on:click=move |_| insert(color)
                                ></button>
                            }
                        })
                        .collect_view()}
                </div>
                <button
                    class="round send"
                    title=move || if clearing.get() { "Clear board" } else { "Send to board" }
                    aria-label=move || if clearing.get() { "Clear board" } else { "Send to board" }
                    disabled=move || !can_send.get()
                    on:pointerdown=|ev| ev.prevent_default()
                    on:click=move |_| submit()
                >
                    <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M10 16V4M4.5 9.5 10 4l5.5 5.5" />
                    </svg>
                </button>
            </div>
        </main>
    }
}

/// List the color tiles that look different from a blank tile.
fn colors() -> impl Iterator<Item = Color> {
    // Black tiles look blank on a black Flagship.
    Color::ALL
        .into_iter()
        .filter(|&color| color != Color::Black)
}

/// List the backgrounds that look different from no background.
fn backgrounds() -> impl Iterator<Item = Background> {
    Background::ALL
        .into_iter()
        .filter(|&background| background != Background::Black)
}

/// Draw the bar under the fill icon in the current background.
fn fill_bar(background: Option<Background>) -> impl IntoView {
    let (class, style) = match background {
        None => ("bar none", String::new()),
        Some(Background::Rainbow) => ("bar rainbow", String::new()),
        Some(solid) => ("bar", format!("--c:{}", css(solid.color(0)))),
    };

    view! { <span class=class style=style></span> }
}

/// Draw a tiny board filled with a background around a bar of text.
fn mini_board(background: Option<Background>) -> impl IntoView {
    let (class, style) = match background {
        None => ("mini none", String::new()),
        Some(Background::Rainbow) => ("mini rainbow", String::new()),
        Some(solid) => ("mini", format!("--c:{}", css(solid.color(0)))),
    };

    view! {
        <span class=class style=style>
            <i></i>
        </span>
    }
}

/// Check whether the main pointer is a mouse or trackpad.
fn has_mouse() -> bool {
    web_sys::window()
        .and_then(|window| window.match_media("(pointer: fine)").ok().flatten())
        .is_some_and(|query| query.matches())
}

/// Capitalize the first letter of a message.
fn sentence(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Read a choice this browser remembers.
fn stored(key: &str) -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item(key)
        .ok()?
}

/// Remember a choice in this browser.
fn store(key: &str, value: &str) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(key, value);
    }
}

/// Read a query parameter from the page URL.
fn param(name: &str) -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;

    UrlSearchParams::new_with_str(&search).ok()?.get(name)
}

/// Send a message to the board server.
async fn send(message: &Message) -> Result<(), String> {
    let request = Request::post("/message")
        .json(message)
        .map_err(|err| err.to_string())?;
    let response = request
        .send()
        .await
        .map_err(|_| "Can't reach the board".to_string())?;
    if response.ok() {
        return Ok(());
    }

    Err(response.text().await.unwrap_or_default().trim().to_string())
}
