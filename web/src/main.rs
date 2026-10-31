mod app;
mod preview;
mod viewport;

fn main() {
    console_error_panic_hook::set_once();
    viewport::fit_to_visual_viewport();
    leptos::mount::mount_to_body(app::App);
}
