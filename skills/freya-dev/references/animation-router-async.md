# Animation, routing, and async integration

## Animation

Enable via the prelude (`freya::animation` is re-exported). Two hooks cover almost everything.

### `use_animation` — manual control

The callback receives `&mut AnimConfiguration` and returns the animated value:

```rust
use freya::prelude::*;

fn app() -> impl IntoElement {
    let mut anim = use_animation(|_| AnimColor::new((246, 240, 240), (205, 86, 86)).time(400));

    rect()
        .background(&*anim.read())
        .width(Size::px(120.))
        .height(Size::px(120.))
        .on_press(move |_| anim.start())
}
```

The callback always receives `&mut AnimConfiguration` — this signature is the same in 0.4 and
0.5, so `use_animation(|conf| ...)` is correct for both. A zero-argument closure
(`use_animation(move || ...)`) belongs to the much older Dioxus-era API (0.1–0.3) and will not
compile.

`UseAnimation<T>` derefs to `State<T>`, so `anim.read()` gives you the current value.
Control methods: `.start()`, `.reverse()`, `.reset()`, `.finish()`, `.run(AnimDirection)`;
state accessors: `.get()`, `.direction()`, `.is_running()`, `.has_run_yet()`, `.runs()`.

### Available animated values

| Type | Built with | Read as |
|---|---|---|
| `AnimColor` | `AnimColor::new(from, to)` | `Color` |
| `AnimNum` | `AnimNum::new(from: f32, to: f32)` | `f32` (via `value()` or `From`) |
| `AnimSequential<A, N>` | `AnimSequential::new([a, b, c])` | the current sub-animation |

Common builders on `AnimColor`/`AnimNum`: `.time(u64)` (milliseconds), `.ease(Ease)`.
`Ease::{In, Out (default), InOut}`. `AnimNum::function(impl Into<Function>)` for a custom curve.

`AnimSequential` chains values so each runs after the previous finishes — good for
"slide in, then fade" style sequences.

### Configuring behaviour

```rust
let mut anim = use_animation(|conf| {
    conf.on_creation(OnCreation::Run);          // start immediately on mount
    conf.on_finish(OnFinish::reverse());        // bounce back when done
    conf.on_change(OnChange::Rerun);            // rerun when dependencies change
    AnimNum::new(0., 100.).time(300)
});
```

- `OnFinish::{Nothing (default), Reverse { delay }, Restart { delay }}` with constructors
  `nothing()`, `reverse()`, `reverse_with_delay(d)`, `restart()`, `restart_with_delay(d)`.
- `OnCreation::{Nothing (default), Run, Finish}`.
- `OnChange::{Reset (default), Finish, Rerun, Nothing}` — note the default is **Reset**, so a
  dependency change resets unless you opt into `Rerun`/`Finish`.

### `use_animation_transition` — reactive, declarative

When you want "animate to the new value whenever this changes", this is the right hook. It
re-runs automatically when the tracked value changes and starts on creation:

```rust
let active = use_state(|| false);

let color = use_animation_transition(*active.read(), |from: Color, to| {
    AnimColor::new(from, to).time(300)
});

rect()
    .background(&*color.read())
    .width(Size::px(160.))
    .height(Size::px(160.))
```

Pass the *value* (read inside `render`), and the callback receives `(from, to)`.

### Explicit dependencies

`use_animation_with_dependencies(&deps, |conf, dep| ...)` when the animation depends on
non-reactive data; states read inside the callback are ignored, so pass them as dependencies.

### Practical notes

- Animate by feeding the animated value into a builder each render: `.background(&*anim.read())`,
  `.width(Size::px(anim.read().value()))`, `.opacity(...)`.
- Animations drive redraws themselves; do not call `Platform::get().send(UserEvent::RequestRedraw)`
  manually.
- Because hook state resets under hot reload, animated values snap back — expected during
  iteration.

## Routing

Enable with `features = ["router"]`.

```rust
use freya::prelude::*;
use freya::router::prelude::*;

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)]
        #[route("/")]
        Home,
        #[route("/settings")]
        Settings,
}

fn app() -> impl IntoElement {
    Router::<Route>::new(|| RouterConfig::default().with_initial_path(Route::Home))
}

#[derive(PartialEq)]
struct AppLayout;

impl Component for AppLayout {
    fn render(&self) -> impl IntoElement {
        rect()
            .expanded()
            .child(Outlet::<Route>::new())
    }
}

#[derive(PartialEq)]
struct Home;

impl Component for Home {
    fn render(&self) -> impl IntoElement {
        Button::new()
            .on_press(|_| {
                let _ = RouterContext::get().push(Route::Settings);
            })
            .child("Go to settings")
    }
}
```

Key pieces:

- `#[derive(Routable, Clone, PartialEq)]` on an enum; `#[route("/path")]` per variant and
  optional `#[layout(LayoutComponent)]` on a group of variants.
- `Router::<Route>::new(|| RouterConfig::default().with_initial_path(...))` mounts the router.
- `Outlet::<Route>::new()` renders the currently matched route — place it inside your layout.
- `Link::new(Route::X)` for declarative navigation.
- `use_route::<Route>()` to read the current route (panics if not under a `Router`).

### Navigating imperatively

`RouterContext::get()` gives the router:

| Method | Effect |
|---|---|
| `.push(target)` | push a new location (back becomes available) |
| `.replace(target)` | replace current location (no back entry) |
| `.go_back()` / `.go_forward()` | history movement |
| `.can_go_back()` / `.can_go_forward()` | history availability |
| `.current::<Route>()` | the active route |
| `.full_route_string()` | the current path |
| `.site_map()` | the registered route tree |

`push`/`replace` return `Result<(), ExternalNavigationFailure>` — navigation can fail when the
target is an external URL that cannot be opened, so `let _ = ...` or handle it.

`NavigationTarget` is `Internal(Route)` or `External(String)`; routes and strings convert into
it, so `.push(Route::Home)` and `.push("/settings")` both work.

### Nested / child routers

For a router inside a route, consume the child route mapping (`consume_child_route_mapping`) and
define the child enum separately; `RouterContext::current` resolves the child route through the
parent's mapping automatically.

## Async integration with Tokio

Freya has its own async runtime, and some ecosystem crates (`reqwest`, `sqlx`, `tonic`)
require a Tokio runtime. Bridge them in `main`:

```rust
use freya::prelude::*;
use tokio::runtime::Builder;

fn main() {
    let rt = Builder::new_multi_thread().enable_all().build().unwrap();
    let _rt = rt.enter();   // keep alive for the whole program

    launch(LaunchConfig::new().with_window(WindowConfig::new(app)))
}
```

**`#[tokio::main]` is explicitly discouraged** — it interferes with Freya's event loop. Use the
manual runtime + `enter()` guard instead, and keep the guard bound to a variable in `main` so it
lives for the whole program.

With the Tokio context active you can use `tokio::sync` (mpsc, watch, broadcast, oneshot) and
`tokio::time` (sleep, interval, timeout) inside Freya's `spawn`/`use_future`.

### The threading rule

Freya's reactivity is single-threaded. You **cannot** update component state from another thread;
`tokio::spawn` cannot touch state because the closures would need `Sync`/`Send` bounds. So:

- UI-updating work → Freya's `spawn()` / `spawn_in_window()` / `spawn_global()`.
- Pure background work (no UI) → `tokio::spawn` is fine.
- Blocking work → `freya::prelude::thread(closure).await`, which moves it off the event loop.

### Watch channels

With `features = ["sdk-tokio"]`, `use_track_watcher(&rx)` re-renders when a `tokio::sync::watch`
value changes, letting a producer live on any thread:

```rust
use freya::sdk::use_track_watcher;

impl App for MyApp {
    fn render(&self) -> impl IntoElement {
        use_track_watcher(&self.rx);
        label().text(self.rx.borrow().to_string())
    }
}
```

`LaunchConfig::with_task(|_| async move { ... })` is where such producers are usually started.

## Combining the three

A realistic pattern: a routed app whose page fetches data on mount, animates a loading
placeholder, and refreshes on a timer.

```rust
#[derive(PartialEq)]
struct Profile { id: u32 }

impl Component for Profile {
    fn render(&self) -> impl IntoElement {
        let task = use_future(move || {
            let id = self.id;                 // captured, stable
            async move { load_profile(id).await }
        });

        let opacity = use_animation_transition(task.state().is_loading(), |from, to| {
            AnimNum::new(from, to).time(200)
        });

        rect()
            .opacity(opacity.read().value())
            .child(match &*task.state() {
                FutureState::Fulfilled(Ok(p)) => p.name.clone(),
                FutureState::Fulfilled(Err(_)) => "Failed to load".to_string(),
                _ => "Loading...".to_string(),
            })
    }
}
```

Note `FutureState<Result<T, E>>` — since `FutureState` has no error variant, wrap the result.
