# Migrating from Freya 0.4 to 0.5

This guide is based on a **direct diff of the published crates** (`freya 0.4.1` vs `freya 0.5.0`
and their `freya-core` counterparts), not on folklore. Where a change is not listed here, the
API is very likely unchanged.

The headline: **0.5 is a refinement of 0.4, not a rewrite.** The reactive core — `use_state`,
`State`, hooks, elements, `use_animation`, `Content`/`Size`/`Gaps`, events — carries over almost
intact. Most breakage comes from the rendering/platform layer, theming internals, component
churn, and crate renames.

## 1. What did *not* change

Good news first — these are byte-for-byte compatible in signature:

- `fn main() { launch(LaunchConfig::new().with_window(WindowConfig::new(app))) }`
- `use_state(|| ..)`, `State::{read, write, set, create}`, `WritableUtils`
- `rect()`, `label()`, `paragraph()`, `image()`, and all extension-trait builders
  (`width`/`height`/`padding`/`spacing`/`background`/`on_press`/…)
- `Size`, `Content`, `Gaps`, `Alignment`, `Position`, `Direction`, `VisibleSize`
- `use_animation(|conf| ...)`, `use_animation_transition`, `AnimColor`, `AnimNum`, `Ease`
- Component model: `#[derive(PartialEq)] struct X; impl Component for X`, `ComponentOwned`, `App`
- Hook set: `use_hook`, `use_drop`, `use_memo`, `use_side_effect`, `use_future`, `use_reactive`,
  `use_provide_context`, `use_consume`, `spawn`, context functions
- `Readable` / `Writable`, `IntoReadable` / `IntoWritable`

So a simple 0.4 app may compile on 0.5 with only a `Cargo.toml` bump. Start there, then fix
what the compiler tells you.

## 2. `Platform` gained a window layer

This is the most common real breakage.

**0.4** — `Platform` was a single handle:

```rust
Platform::get().send(UserEvent::RequestRedraw);
```

**0.5** — `Platform` is now app-level, and per-window operations live on `PlatformWindow`:

```rust
Platform::get().current_window().request_redraw();
```

New/changed surface on `Platform`: `detect()`, `is_desktop()`, `is_mobile()`, `new(..)`,
`window(id)`, `try_window(id)`, `current_window()`, `register_window(..)`,
`unregister_window(..)`, `send(GlobalUserEvent)`, `open_url(..)`, `exit()`,
`load_font(name, bytes)`.

`PlatformWindow` provides `send(UserEvent)`, `request_redraw()`, `set_custom_scale_factor(f64)`,
`window()`-scoped state such as `focused_accessibility_id` and `navigation_mode`.

`send` also changed type: app-level `Platform::send` takes `GlobalUserEvent`, not `UserEvent`.

Practical fix: replace `Platform::get().send(UserEvent::…)` with
`Platform::get().current_window().send(UserEvent::…)`, and use `.request_redraw()` rather than
sending a redraw event.

## 3. Metrics/performance plugin renamed

- 0.4: `freya_performance_plugin::PerformanceOverlayPlugin`
- 0.5: `freya::metrics::MetricsPlugin` (feature `metrics`), auto-added in debug builds

```toml
freya = { version = "0.5", features = ["metrics"] }
```

```rust
LaunchConfig::new().with_plugin(
    freya::metrics::MetricsPlugin::default().with_visible_performance(true),
)
```

The 0.4 `performance` feature is gone; `vulkan` is also gone as a feature — renderer choice is
now `RendererPreference::Vulkan` / `FREYA_RENDERER=vulkan`.

## 4. Theming internals became private

0.5 no longer re-exports the theming internals (`extensions::*`, `themes::*`, and the internal
`prelude::*`) from `freya::components`. Import the public names instead:

```rust
// 0.5
use freya::prelude::{Theme, ColorsSheet, light_theme, dark_theme, Preference, define_theme, get_theme};
```

Public theming surface now: `Theme`, `ColorsSheet`, `light_theme()`, `dark_theme()`,
`LIGHT_COLORS` / `DARK_COLORS`, `Preference`, `define_theme!`, `get_theme!`,
`get_theme_or_default!`, `use_init_theme`, `use_provide_theme`, `use_theme`,
`get_theme_or_default`, `FromPreference::to_theme`.

Also note `ColorsSheet` has **no `hover` field** (use `active`/`focus`), and
`Theme::default()` is `light_theme()`.

## 5. Component churn

Removed in 0.5 (were in 0.4):

- `canvas::*` (`Canvas`)
- `cursor_area::*` (`CursorArea`)

If you used `Canvas` for custom drawing, use a `RenderCallback` background (or the `plot`
feature with Plotters) instead.

Renamed/moved modules:

- `markdown` moved out of `freya-components` into its own `freya-markdown` crate; the
  `markdown` feature still re-exports it as `freya::markdown::*`.
- `webview` (`freya-webview`) was dropped in favour of `html` (`freya-html`, Blitz-based).
  Add `features = ["html"]` and use **`HtmlViewer`** (not `HtmlView`, which does not exist):

  ```rust
  use freya::html::*;

  let handle = use_html(|| HtmlSource::url("https://example.com"));
  HtmlViewer::new(handle).width(Size::px(400.)).height(Size::px(300.));
  ```

  `HtmlViewer::new(handle: HtmlHandle)`, plus `.font(Bytes)`. It also implements `ContainerExt`
  and `LayoutExt`, so `.padding(..)`, `.width(..)` etc. work. `use_html` returns the
  `HtmlHandle`, which drives navigation (back/forward/current URL) and also accepts
  `HtmlSource::html("...")` for inline documents.

New in 0.5 (likely useful in a migration): `borderless` (`BorderlessPlugin`), `html`,
`camera`, `video`, `query`, `zoom-shortcuts`, `titlebar` (unchanged name, new component set),
`virtualization` improvements in `VirtualScrollView`.

## 6. Core-level renames to watch for

From the `freya-core` diff, functions present in 0.4 but gone in 0.5:

| 0.4 | 0.5 replacement |
|---|---|
| `spawn_forever` | `spawn_global` (or `spawn_in_window`) |
| `FutureState::{as_fulfilled, try_as_fulfilled}` | `FutureState::ok()` / `.unwrap()` |
| `animation_clock::new` | `AnimationClock::get()` |
| `rect().blur(f32)` | `rect().backdrop_blur(f32)` for backgrounds; `image().blur(f32)` still exists via `ImageExt` |
| `image().corner_radius(..)` | `.corner_radius(..)` via `DecorationExt` (works on `ImageViewer`) |

New convenience worth adopting:

- `spawn_in_window`, `spawn_in_scope`, `TaskHandle::is_finished`
- `State::{try_read, try_peek, read_unchecked, peek_unchecked, try_write, try_write_unchecked}`
- `use_side_effect_value`, `Effect::create_global`, `GlobalContexts::*`
- `freya::prelude::thread(closure).await` for blocking work
- `Color::{luminance, brightness, is_dark, is_light, with_af32}`
- `CornerRadius::{top_left, top_right, bottom_right, bottom_left, smoothing}`
- `Paragraph::{visible_location, cursor_style_data}`
- `events::{div, is_device}`, `EventsCombos::{moved, released}`

## 7. Router

The router crate is versioned `0.4.1` even inside Freya 0.5 (expect it to bump separately), and
its API is what 0.5 ships. Two things to check:

- Mount with `Router::<Route>::new(|| RouterConfig::default().with_initial_path(..))`.
  A `router::<Route>()` helper is also surfaced by components in some docs; prefer the
  `Router` component form, which is what the router crate exposes.
- `Outlet::<Route>::new()` inside a layout component renders the matched route.

`#[derive(Routable, Clone, PartialEq)]`, `#[route("/path")]`, `#[layout(Comp)]`, `Link::new`,
`RouterContext::get()` with `.push/.replace/.go_back/.go_forward/.current::<R>()` are stable.

## 8. Radio

`freya-radio` 0.5 API: `RadioChannel<T>` with `derive_channel(self, &T) -> Vec<Self>`,
`use_init_radio_station`, `use_radio`, `use_radio_station`, `use_share_radio`,
`Radio::{read, write, with, write_with, write_channel, write_silently, slice*}`,
`ChannelSelection::{Current, Select, Silence}`, `DataReducer` + `RadioReducer::apply`.

If 0.4 code used a different slice/reducer surface, prefer rewriting to this shape — it is
small and stable.

## 9. Migration procedure

1. **Bump the version and nothing else.** `freya = "0.5"`. Run `cargo check`.
2. **Fix crate/feature names first** (`webview` → `html`, `performance` → `metrics`, `vulkan`
   feature → `RendererPreference`). Feature renames cascade into `cfg` blocks; get them out of
   the way early.
3. **Fix `Platform` calls.** Almost every app touched it. Convert to
   `Platform::get().current_window()...`.
4. **Fix removed components** (`Canvas`, `CursorArea`) and removed theming imports.
5. **Rebuild the UI incrementally**, starting from a screen that compiles and runs, not from a
   full rewrite. Because the builder API is stable, most files port unchanged.
6. **Re-verify state code** for the guard/`peek` pitfalls described in `hooks-and-state.md` —
   these were latent in 0.4 too but surface more often now.
7. **Re-run the launch flow** and confirm a window appears before debugging layout.

## 10. Do not trust recalled 0.4 patterns

Some blog posts and older examples use APIs that predate even 0.4 (the Dioxus-era 0.1–0.3
design): `RenderElement`, `Fuse`, `Event::new`, `BuiltinStyle`, `LaunchBuilder`, `use_node`,
`Signal`. None of these exist in 0.5, and several did not exist in 0.4 either. If you see them
in a snippet, the snippet is stale — translate it to the element + builder model instead of
trying to import it.
