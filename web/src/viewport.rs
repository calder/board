use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::HtmlElement;

/// Keep the page sized to the area an on-screen keyboard leaves visible.
pub fn fit_to_visual_viewport() {
    fit();
    let Some(window) = web_sys::window() else {
        return;
    };
    let listener = Closure::<dyn Fn()>::new(fit);
    if let Some(viewport) = window.visual_viewport() {
        let _ =
            viewport.add_event_listener_with_callback("resize", listener.as_ref().unchecked_ref());
    }
    let _ = window.add_event_listener_with_callback("resize", listener.as_ref().unchecked_ref());
    listener.forget();
}

fn fit() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let height = match window.visual_viewport() {
        Some(viewport) => viewport.height(),
        None => window
            .inner_height()
            .ok()
            .and_then(|h| h.as_f64())
            .unwrap_or(0.0),
    };
    let root = window
        .document()
        .and_then(|document| document.document_element())
        .and_then(|element| element.dyn_into::<HtmlElement>().ok());
    if let Some(root) = root {
        let _ = root
            .style()
            .set_property("--app-height", &format!("{height}px"));
    }
    window.scroll_to_with_x_and_y(0.0, 0.0);
}
