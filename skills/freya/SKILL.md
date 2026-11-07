---
name: freya
description: >-
  Freya 0.5 (Rust native GUI on Skia) development. Use this whenever the user works on a Freya
  app, mentions freya/rect/label/use_state/Button/ScrollView in a Rust GUI context, hits a Freya
  build or environment error (linker, skia, clang, X11/Wayland, windows-gnu), needs a desktop GUI
  in Rust, needs hot reload or window configuration, or is porting a Freya 0.4 app to 0.5. Covers
  scaffolding a working Cargo.toml, run/debug/hot-reload, layout and styling, hooks and state,
  events, animation, routing, and diagnosing real failures.
---

# Freya 0.5 development

Freya is a declarative, cross-platform native GUI library for Rust, rendered with Skia.
Version `0.5` was released 2026-11-05. It is a refinement of 0.4 rather than a rewrite, but it
moved enough surface (`Platform`, theming internals, a few components and crate names) that
old code cannot be assumed to work unmodified.

## Ground rules for this skill

This skill has three kinds of content, and you should reach for them in this order:

1. **This file** — the decisions that most often go wrong. Read it first.
2. **`references/`** — exact signatures. Read the matching file before writing code that
   depends on details you are not certain of. Do not guess a method name.
3. **`examples/`** — complete projects you can copy. Copying a known-good skeleton is far
   cheaper than debugging API drift.

Two things to internalize because they cause most wasted turns:

- **Freya 0.4 → 0.5 is mostly a refinement, not a rewrite** — but it does break. The reactive
  core (`use_state`, `State`, hooks, elements, animation) is almost identical, so 0.4 UI code
  often ports with small edits. What moved: `Platform` gained a `current_window()` layer,
  theming internals became private, several components were removed or renamed, and some crates
  were renamed. Do not assume: read `references/migration-0.4-to-0.5.md`, which lists the
  verified deltas rather than guesses.
- **The upstream project ships its own AI guidance** (`skills/freya/SKILL.md` and
  <https://freyaui.dev/llms.txt>). This skill is complementary: it is stronger on environment
  setup, verified signatures, and failure diagnosis. When they disagree on a signature, trust
  this skill's `references/`, which were read from the 0.5.0 crate source.

## Do not guess: verify

Every symbol used by this skill was checked against the `freya 0.5.0` crate sources. If you
need an API that is not in `references/`, prefer looking it up rather than inventing it:

```sh
cargo doc -p freya --no-deps --open          # local, already resolved
# or inspect the actual source:
#   $CARGO_HOME/registry/src/<registry>/freya-0.5.0/src/
```

Guessing a builder method produces a compile error loop that costs more than the lookup.

## Step 1: a project that actually builds

Minimal, correct `Cargo.toml` (edition 2024, as Freya 0.5 itself uses):

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"

[dependencies]
freya = "0.5"

[features]
# Forwards to Freya's hot reload; keeps it out of release builds.
hotreload = ["freya/hotreload"]
```

`src/main.rs`:

```rust
use freya::prelude::*;

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(app).with_size(600., 400.)))
}

fn app() -> impl IntoElement {
    rect()
        .expanded()
        .center()
        .background((30, 30, 30))
        .color(Color::WHITE)
        .child("Hello, Freya")
}
```

```sh
cargo run
```

If this does not show a window, stop and fix the environment before writing more UI. The
compiler and linker failures below are far more common than mistakes in your UI code, and
they are all mechanical.

### The three failures that actually happen

**1. `clang` / `libclang` / `skia` build errors.** Freya builds Skia from source through
`skia-safe`, which needs LLVM. Set `LIBCLANG_PATH` to the directory *containing* the
`libclang` library (not its parent, and not the binary path).

```powershell
# Windows (PowerShell)
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
```

```sh
# Linux / macOS
export LIBCLANG_PATH=/usr/lib/llvm-18/lib
```

On Windows prefer the `msvc` toolchain (`rustup default stable-x86_64-pc-windows-msvc`) with
Visual Studio's "Desktop development with C++" workload. The `windows-gnu` toolchain has
known compile problems — switch toolchains rather than fighting the linker.

**2. First build appears hung.** Skia compilation legitimately takes several minutes and
dominates the initial build. Start the first `cargo build` early in the session and keep
working; do not interpret slow progress as a failure. Subsequent builds are fast.

**3. Linux missing system libraries.** Debian/Ubuntu:

```sh
sudo apt install build-essential libssl-dev pkg-config cmake libclang-dev libudev-dev \
  libasound2-dev liblzma-dev libfontconfig1-dev libfreetype6-dev libegl1-mesa-dev \
  libgl1-mesa-dev libgles2-mesa-dev libwayland-dev
```

`mold` as a linker is not supported by Freya 0.5. macOS needs no setup. See
`references/setup-and-troubleshooting.md` for the full platform matrix and error-to-fix table.

## Step 2: the mental model

Freya is declarative and reactive, similar in spirit to React:

- **`main`** calls `launch(...)` with a `WindowConfig`.
- **The root component** is a plain function `fn app() -> impl IntoElement`, or a struct
  implementing `App` when you need to pass data in from `main`.
- **Elements** are built with a fluent builder: `rect()`, `label()`, `paragraph()`, `image()`.
  Layout and styling are *methods on the element*, not CSS.
- **Components** are structs implementing `Component`. They are state boundaries: when state
  changes, only components subscribed to it re-render.
- **Hooks** (`use_state`, `use_memo`, …) attach state to a component and are called at the top
  of `render`.

```rust
use freya::prelude::*;

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(app)))
}

fn app() -> impl IntoElement {
    let mut count = use_state(|| 0);

    rect()
        .expanded()
        .center()
        .spacing(8.)
        .child(label().text(format!("Clicked {} times", count.read())))
        .child(Button::new().on_press(move |_| *count.write() += 1).child("Click me"))
}
```

### Component vs function: pick deliberately

| You need… | Use |
|---|---|
| The app root | plain `fn app() -> impl IntoElement` |
| Reusable UI with hooks or props | `#[derive(PartialEq)] struct X; impl Component for X` |
| Stateless snippet, no hooks | plain function returning `impl IntoElement` (utility fn) |
| `render` needing to move fields into closures | `impl ComponentOwned` with `#[derive(Clone)]` |

A hook-using helper written as a plain function is the most common structural mistake: it
cannot be keyed, cannot take reactive props, and silently re-runs on every parent render.
Anything with hooks that is reused must be a `Component`.

```rust
#[derive(PartialEq)]
struct Counter {
    initial: i32,
}

impl Component for Counter {
    fn render(&self) -> impl IntoElement {
        let mut count = use_state(|| self.initial);
        Button::new()
            .on_press(move |_| *count.write() += 1)
            .child(format!("Count: {}", count.read()))
    }
}
```

`#[derive(PartialEq)]` is load-bearing: Freya compares props to decide whether to skip
re-rendering. Omitting it does not fail to compile in every case, but it disables re-rendering
optimization.

## Step 3: layout is the second most common source of confusion

Freya's layout engine (Torin) is flex-like but the names differ from CSS. Learn these four
distinctions before writing any real screen:

1. **`Size::fill()` vs `Size::flex(n)`.** `fill()` takes the remaining space along the parent's
   main axis and can cause later siblings to overflow. `flex(n)` shares space *between*
   siblings by weight, and **requires the parent to set `.content(Content::Flex)`**. For anything
   resembling a real layout, `flex` is what you want.
2. **`.expanded()`** is a shorthand for filling the remaining space; handy for "make this page
   fill the window".
3. **`.horizontal()` / `.vertical()`** set the direction. Prefer them over `Direction::_`.
4. **`.center()`** centers children on both axes; `.main_align()`, `.cross_align()` and
   `.spacing()` handle the rest. Default alignment is `Start`.

```rust
// Header on top, body fills the rest, footer at the bottom
rect()
    .expanded()
    .content(Content::Flex)
    .spacing(8.)
    .padding(12.)
    .child(label().text("Header"))
    .child(rect().height(Size::flex(1.)).child("Body"))
    .child(label().text("Footer"))
```

Units, gaps, alignment values, borders, shadows, gradients, opacity/rotation/scale and
scroll behaviour are catalogued in `references/layout-and-styling.md`.

## Step 4: state — choose the narrowest tool

| Situation | Use |
|---|---|
| Data owned by one component | `use_state` |
| Derived value, recompute only when inputs change | `use_memo`, or just compute in `render` |
| App-wide state needing surgical updates | Freya Radio (feature `radio`) |
| Read-only / write-only prop that hides its storage | `Readable<T>` / `WritableSetter<T>` |
| Dependency injection across the tree | context (`use_provide_context` / `use_consume`) |
| Remote/server data with caching + refetch | `freya-query` (feature `query`) |

`State<T>` is `Copy`, so closures capture it directly without `.clone()`:

```rust
let mut count = use_state(|| 0);
rect().on_press(move |_| *count.write() += 1).child(count.read().to_string())
```

Two habits that prevent real bugs:

- **Prefer `set_if_modified`** when the new value might equal the old; it skips the write and
  avoids waking subscribers.
- **Let read/write guards drop naturally.** Prefer `let n = *count.read();` over holding a
  guard across other state access; holding a guard while writing the same state deadlocks or
  panics.

`Input::new` takes `Into<WritableSetter<String>>` and `Switch::new` takes
`Into<WritableSetter<bool>>`, so pass state directly:

```rust
let name = use_state(String::new);
let toggled = use_state(|| false);
rect().child(Input::new(name).placeholder("Name")).child(Switch::new(toggled))
```

Details, including the Radio channel/reducer model, are in `references/hooks-and-state.md`.

## Step 5: hooks rules are not stylistic

Hooks must run in the same order on every render. That means **never** call a hook inside a
condition, a loop, an event handler, or an async block. Capture hook values into `move`
closures instead of calling hooks there.

```rust
// correct: hook at top level, state captured by the closure
let mut open = use_state(|| false);
Button::new().on_press(move |_| *open.write() = !*open.read()).child("Toggle");
```

Inside async code you cannot call hooks; use the non-hook counterparts such as
`consume_context::<T>()` instead of `use_consume::<T>()`.

## Step 6: events

Attach handlers to elements, not components. `Event<T>` offers two different cancellation
tools that are easy to confuse:

- `.stop_propagation()` — stop **this** event from bubbling to ancestor handlers.
- `.prevent_default()` — cancel the **related follow-up** events (for example, `on_mouse_up`
  calling `prevent_default()` suppresses the `on_pointer_press` and `on_global_pointer_up`
  that would otherwise fire).

Prefer **`on_press`** over raw mouse events for anything interactive: it covers click, tap, and
Enter/Space when focused, so keyboard accessibility comes for free. Reach for
`on_mouse_*` / `on_pointer_*` only for pointer-specific behaviour such as dragging.

To expose a callback from your own component, take an `EventHandler<T>` field (it is `Copy`):

```rust
#[derive(PartialEq)]
struct Confirm {
    on_accept: EventHandler<()>,
}

impl Component for Confirm {
    fn render(&self) -> impl IntoElement {
        let on_accept = self.on_accept;
        Button::new().on_press(move |_| on_accept.call(())).child("OK")
    }
}
```

Full catalog, propagation order (capture → leave → enter → rest), and click-combo detection:
`references/events.md`.

## Step 7: dynamic lists need keys

Any list that can be reordered, inserted into, or deleted from needs a stable `.key()`.
Without it Freya reconciles by index and local state jumps to the wrong row.

```rust
ScrollView::new().children(
    items.read().iter().map(|item| label().key(item.id).text(item.title.clone())),
)
```

Key by a stable identity (id), not the index, whenever items can move.

## Step 8: run, iterate, debug

```sh
cargo run                 # normal run
cargo run --release       # release build
cargo test                # headless component tests via freya-testing
cargo clippy --all-targets
```

**Renderer selection** — if the window is blank, flickering, or GPU initialization fails, force
a backend with the `FREYA_RENDERER` environment variable before suspecting your UI code:

```sh
FREYA_RENDERER=software cargo run     # always works, slowest
FREYA_RENDERER=opengl cargo run
```

**Debug overlays** are built in for debug builds: `Ctrl+Shift+P` toggles the performance overlay,
`Ctrl+Shift+L` toggles stats (not available in release). Enable the `metrics` feature to use
them deliberately.

**Hot reload** uses the Dioxus CLI and only patches function bodies:

```sh
cargo install dioxus-cli
dx serve --hot-patch --features hotreload
```

Hook state is **reset to initial values** on every patch, and tasks spawned with `spawn` are
cancelled. Structural changes (new functions, types, trait impls, changed signatures, struct
layout, `const`/`static`) cannot be patched — restart instead. Plan for this: put long-lived
state in places that survive, and expect to re-enter UI state while iterating.

**Headless testing** with `freya-testing` is the cheapest way to verify behaviour:

```rust
use freya_testing::prelude::*;

let mut runner = launch_test(app);
runner.sync_and_update();
runner.click_cursor((30., 80.));
```

## Step 9: features, and the cost of enabling them

Default features are `winit`, `gpu`, `accessibility`, `platform-text-gamma`. Add only what you
use; every feature pulls dependencies and build time.

```toml
freya = { version = "0.5", features = ["router", "radio", "icons"] }
```

Frequently used: `router`, `radio`, `query`, `i18n`, `icons` (Lucide), `material-design`,
`markdown`, `plot`, `html`, `code-editor`, `terminal-pty`, `camera`, `video`, `titlebar`,
`borderless`, `devtools`, `gif`, `calendar`, `remote-asset`, `zoom-shortcuts`.

For software-only builds without the GPU stack:

```toml
freya = { version = "0.5", default-features = false, features = ["winit"] }
```

Prefer starting with defaults, then trimming once the app works — removing `accessibility` or
`gpu` too early removes the tooling you need to debug.

## Reference map

| File | Read it when |
|---|---|
| `references/setup-and-troubleshooting.md` | build fails, window blank, platform dependencies, error→fix table |
| `references/layout-and-styling.md` | writing layout, sizes, colors, borders, shadows, gradients, scrolling |
| `references/hooks-and-state.md` | `use_state`/`use_memo`/context/Radio/`Readable`/`Writable`, async |
| `references/events.md` | choosing handlers, propagation, keyboard/focus, file drop |
| `references/animation-router-async.md` | `use_animation`, transitions, routing, threads and Tokio |
| `references/components.md` | built-in component constructors (`Button`, `Input`, `ScrollView`, …) |
| `references/migration-0.4-to-0.5.md` | porting an existing Freya app |
| `examples/minimal/` | the smallest correct complete project |
| `examples/counter/` | state + components + events |
| `examples/todo/` | lists, keys, text input, derived state, tests |
| `examples/routing/` | routed multi-page app |
| `examples/animated/` | animation and theming |

## Working style

- Run `cargo check` early and often; Freya's type errors are usually precise.
- When something does not render, first confirm the element has a size (elements sized
  `Inner`/auto with no content can collapse to zero), then check `Content::Flex` is set on the
  parent if you used `Size::flex`.
- Prefer reading the actual crate source over recalling API from memory. In this domain,
  recall is wrong far more often than it is right.
