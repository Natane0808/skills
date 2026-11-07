use freya::prelude::*;
use freya::router::prelude::*;

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(app).with_size(700., 480.)))
}

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)]
        #[route("/")]
        Home,
        #[route("/settings")]
        Settings,
        #[route("/about")]
        About,
}

fn app() -> impl IntoElement {
    use_init_theme(light_theme);
    Router::<Route>::new(|| RouterConfig::default().with_initial_path(Route::Home))
}

/// Persistent chrome around the routed page. `Outlet` renders the matched route.
#[derive(PartialEq)]
struct AppLayout;

impl Component for AppLayout {
    fn render(&self) -> impl IntoElement {
        rect()
            .expanded()
            .content(Content::Flex)
            .theme_background()
            .theme_color()
            .child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .padding(12.)
                    .spacing(8.)
                    .cross_align(Alignment::Center)
                    .child(label().text("Router demo").font_size(18.))
                    .child(rect().width(Size::flex(1.)).child(""))
                    .child(Link::new(Route::Home).child(Button::new().child("Home")))
                    .child(Link::new(Route::Settings).child(Button::new().child("Settings")))
                    .child(Link::new(Route::About).child(Button::new().child("About"))),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::flex(1.))
                    .padding(16.)
                    .child(Outlet::<Route>::new()),
            )
    }
}

#[derive(PartialEq)]
struct Home;

impl Component for Home {
    fn render(&self) -> impl IntoElement {
        rect()
            .spacing(8.)
            .child(label().text("Home").font_size(24.))
            // Programmatic navigation: push keeps history, replace does not.
            .child(
                Button::new()
                    .on_press(|_| {
                        let _ = RouterContext::get().push(Route::Settings);
                    })
                    .child("Go to settings"),
            )
    }
}

#[derive(PartialEq)]
struct Settings;

impl Component for Settings {
    fn render(&self) -> impl IntoElement {
        let can_back = RouterContext::get().can_go_back();

        rect()
            .spacing(8.)
            .child(label().text("Settings").font_size(24.))
            .child(
                Button::new()
                    .enabled(can_back)
                    .on_press(|_| RouterContext::get().go_back())
                    .child("Back"),
            )
            .child(
                Button::new()
                    .on_press(|_| {
                        let _ = RouterContext::get().replace(Route::About);
                    })
                    .child("Replace with About"),
            )
    }
}

#[derive(PartialEq)]
struct About;

impl Component for About {
    fn render(&self) -> impl IntoElement {
        let route: Route = RouterContext::get().current::<Route>();

        rect()
            .spacing(8.)
            .child(label().text("About").font_size(24.))
            .child(label().text(format!("Current route: {route}")))
    }
}
