use freya::prelude::*;

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(app).with_size(640., 460.)))
}

fn app() -> impl IntoElement {
    let mut theme = use_init_theme(light_theme);
    let is_dark = theme.read().name == "dark";

    rect()
        .expanded()
        .content(Content::Flex)
        .theme_background()
        .theme_color()
        .padding(20.)
        .spacing(16.)
        .child(
            rect()
                .horizontal()
                .spacing(8.)
                .child(
                    Button::new()
                        .on_press(move |_| {
                            theme.set(if is_dark { light_theme() } else { dark_theme() });
                        })
                        .child(if is_dark { "Light mode" } else { "Dark mode" }),
                ),
        )
        .child(rect().height(Size::flex(1.)).center().child(ManualAnimation))
        .child(rect().height(Size::flex(1.)).center().child(TransitionDemo))
}

/// Manual animation: the value moves only when `.start()` / `.reverse()` is called.
#[derive(PartialEq)]
struct ManualAnimation;

impl Component for ManualAnimation {
    fn render(&self) -> impl IntoElement {
        let mut anim = use_animation(|conf| {
            // Start on mount so the box is visible immediately.
            conf.on_creation(OnCreation::Finish);
            AnimColor::new((246, 240, 240), (205, 86, 86)).time(600)
        });

        rect()
            .center()
            .spacing(10.)
            .child(
                rect()
                    .width(Size::px(110.))
                    .height(Size::px(110.))
                    .corner_radius(CornerRadius::new_all(12.))
                    .background(&*anim.read()),
            )
            .child(label().text("Manual: start / reverse").font_size(13.))
            .child(
                rect()
                    .horizontal()
                    .spacing(8.)
                    .child(Button::new().on_press(move |_| anim.start()).child("Start"))
                    .child(Button::new().on_press(move |_| anim.reverse()).child("Reverse")),
            )
    }
}

/// Transition animation: re-runs automatically whenever the tracked value changes.
#[derive(PartialEq)]
struct TransitionDemo;

impl Component for TransitionDemo {
    fn render(&self) -> impl IntoElement {
        let mut active = use_state(|| false);
        let on = *active.read();

        let size = use_animation_transition(on, |from, to| {
            AnimNum::new(if from { 110. } else { 60. }, if to { 110. } else { 60. }).time(350)
        });

        let width = size.read().value();

        rect()
            .center()
            .spacing(10.)
            .child(
                rect()
                    .width(Size::px(width))
                    .height(Size::px(width))
                    .corner_radius(CornerRadius::new_all(12.))
                    .background((80, 140, 240)),
            )
            .child(label().text("Transition: toggle").font_size(13.))
            .child(
                Button::new()
                    .on_press(move |_| active.toggle())
                    .child(if on { "Collapse" } else { "Expand" }),
            )
    }
}
