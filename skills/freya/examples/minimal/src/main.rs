use freya::prelude::*;

fn main() {
    launch(
        LaunchConfig::new()
            .with_window(WindowConfig::new(app).with_size(600., 400.).with_title("Minimal")),
    )
}

fn app() -> impl IntoElement {
    rect()
        .expanded()
        .center()
        .background((30, 30, 34))
        .child(label().text("Hello, Freya").color((240, 240, 240)).font_size(24.))
}
