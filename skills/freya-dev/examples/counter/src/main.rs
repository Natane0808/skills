use freya::prelude::*;

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(app).with_size(420., 300.)))
}

fn app() -> impl IntoElement {
    use_init_theme(dark_theme);

    rect()
        .expanded()
        .center()
        .theme_background()
        .theme_color()
        .spacing(16.)
        .child(Counter::new(0))
}

#[derive(PartialEq)]
struct Counter {
    initial: i32,
}

impl Counter {
    fn new(initial: i32) -> Self {
        Self { initial }
    }
}

impl Component for Counter {
    fn render(&self) -> impl IntoElement {
        let mut count = use_state(|| self.initial);

        let doubled = *count.read() * 2;

        rect()
            .center()
            .spacing(12.)
            .padding(24.)
            .child(label().text(format!("Count: {}", count.read())).font_size(28.))
            .child(label().text(format!("Doubled: {doubled}")).font_size(14.))
            .child(
                rect()
                    .horizontal()
                    .spacing(8.)
                    .child(
                        Button::new()
                            .on_press(move |_| *count.write() -= 1)
                            .child("Decrease"),
                    )
                    .child(
                        Button::new()
                            .on_press(move |_| *count.write() += 1)
                            .child("Increase"),
                    )
                    .child(
                        Button::new()
                            .on_press(move |_| count.set(0))
                            .child("Reset"),
                    ),
            )
    }
}
