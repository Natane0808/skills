# Setup, building, and troubleshooting

Freya 0.5.0. Compiled artifacts are heavy (Skia); most setup pain is toolchain and graphics,
not Rust.

## Platform requirements

| Platform | Graphics backend | Setup |
|---|---|---|
| Windows | OpenGL (default), Software fallback, Vulkan opt-in | Visual Studio 2022 with "Desktop development with C++" |
| macOS | Metal, Software fallback | none |
| Linux | Vulkan preferred, OpenGL fallback, Software last | system dev libraries (below) |
| Android (experimental) | OpenGL | SDK + NDK + `cargo-ndk` |
| Web (experimental) | WebGL | Emscripten + `wasm32-unknown-emscripten` |

### Dependencies

Debian / Ubuntu / PopOS:

```sh
sudo apt install build-essential libssl-dev pkg-config cmake libclang-dev libudev-dev \
  libasound2-dev liblzma-dev libfontconfig1-dev libfreetype6-dev libegl1-mesa-dev \
  libgl1-mesa-dev libgles2-mesa-dev libwayland-dev
```

Arch:

```sh
sudo pacman -S base-devel openssl cmake clang
```

Fedora:

```sh
sudo dnf install openssl-devel clang-devel systemd-devel @development-tools -y
```

Root crates: `libclang` (bindgen), OpenGL/EGL/Wayland (windowing + Skia GL), fontconfig and
freetype (text), ALSA (audio, pulled by dependencies), lzma.

Key packages and why they matter:

- **`libclang` / `build-essential` / `cmake`** — `skia-safe` runs bindgen and a Skia build via
  CMake; without them the build fails early with bindgen or CMake errors.
- **`libwayland-dev`, `libegl1-mesa-dev`, `libgl1-mesa-dev`, `libgles2-mesa-dev`** — windowing
  surface and Skia GPU context creation; missing them compiles but fails at runtime.
- **fontconfig / freetype** — system font discovery; missing them usually still builds but text
  may render blank or with wrong metrics.

## Building Skia for the first time

The first build compiles Skia and can take several minutes (longer on Windows). Treat this as
normal:

- Start `cargo build` early and do other work while it runs.
- Incremental rebuilds afterwards are seconds.
- Do not add `-j1` or similar; let Cargo parallelize.

Ways to avoid paying it repeatedly:

```sh
cargo build --release          # once; then `cargo run --release` is fast
# or, share the target dir across projects
CARGO_TARGET_DIR=$HOME/.cache/freya-target
```

## `LIBCLANG_PATH`

`skia-safe`/bindgen locates `libclang` at build time. The value must be the directory that
contains the library file, not the parent and not the executable:

```powershell
# Windows
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
```

```sh
# Linux / macOS
export LIBCLANG_PATH=/usr/lib/llvm-18/lib      # not .../lib/clang
```

If `clang` is not installed or not on `PATH`, Freya's build scripts rely on the system compiler.
On Windows install LLVM (or rely on the VS C++ toolchain) and confirm:

```sh
clang --version
```

## Toolchain choice on Windows

Use the MSVC toolchain. `windows-gnu` has recurring compile problems with Freya's dependency
tree (notably around C++ and Skia); switching is faster than debugging it.

```sh
rustup default stable-x86_64-pc-windows-msvc
rustup component add clippy rustfmt
```

If you must stay on `windows-gnu`, expect to work around link-time C++ issues yourself.

## Linkers

`mold` is explicitly **not supported** by Freya 0.5. Remove it from `.cargo/config.toml`
(`-C link-arg=-fuse-ld=mold`) if you inherit such a config; symptom is link failures or crashes
at startup.

## Renderer selection

Platform default is chosen automatically. Force one with `FREYA_RENDERER`:

```sh
FREYA_RENDERER=software cargo run    # CPU; always works, slowest
FREYA_RENDERER=opengl cargo run
FREYA_RENDERER=vulkan cargo run      # Linux/Windows (Windows is opt-in)
```

Or per window:

```rust
WindowConfig::new(app).with_renderer(RendererPreference::Software)
```

`RendererPreference::{Auto, Software, OpenGl, Vulkan}`. `Auto` respects `FREYA_RENDERER` before
applying platform defaults. GPU backends require the `gpu` feature.

Software-only build (no GPU stack, smaller build):

```toml
freya = { version = "0.5", default-features = false, features = ["winit"] }
```

## Launch configuration reference

```rust
launch(
    LaunchConfig::new()
        .with_window(
            WindowConfig::new(app)               // or WindowConfig::new_app(AppStruct{..})
                .with_size(900., 600.)
                .with_min_size(400., 300.)
                .with_title("My App")
                .with_resizable(true)
                .with_decorations(true)
                .with_transparency(false)
                .with_background((24, 24, 27))
                .with_custom_scale_factor(1.0)
                .with_root_context(AppConfig::default())
                .with_renderer(RendererPreference::Auto)
        )
        .with_font("Inter", Bytes::from_static(include_bytes!("../assets/Inter.ttf")))
        .with_default_font("Inter")
        .with_gpu_resource_cache_limit(GpuResourceCacheLimit::Normal)
        .with_exit_on_close(true)
)
```

Available `LaunchConfig` builders: `with_window`, `with_tray`, `with_plugin`, `with_font`,
`with_fallback_font`, `with_default_font`, `with_global_context`, `with_exit_on_close`,
`with_task`, `with_event_loop`, `with_gpu_resource_cache_limit`.

`GpuResourceCacheLimit::{Small, Normal (256 MiB), Large}` or `from_mb(n)`. Lower it for
GPU-memory-constrained machines; raise it for image-heavy apps.

## Hot reload

```sh
cargo install dioxus-cli
dx serve --hot-patch --features hotreload
```

Requires a feature in your crate forwarding to Freya:

```toml
[features]
hotreload = ["freya/hotreload"]
```

For an example target: `dx serve --hot-patch --features hotreload --example my_example`.

What happens on every patch:

1. All tasks spawned with `spawn` are cancelled.
2. Every scope's hook state resets to its initial value.
3. Every scope is marked dirty and re-renders.

So hot reload preserves the *process*, not the *state*. Design for it: keep expensive or
long-lived work outside hook state, and do not be surprised when a form clears.

Changes that **cannot** be patched (restart the app): adding/removing functions, types or
trait impls, changing function signatures, changing struct layout, changing `const`/`static`
initializers.

## Debug overlays

- Performance overlay: `Ctrl+Shift+P` (`Cmd+Shift+P` on macOS).
- Stats overlay: `Ctrl+Shift+L` (`Cmd+Shift+L`) — debug builds only.
- Enable the `metrics` feature to control them programmatically:

```toml
freya = { version = "0.5", features = ["metrics"] }
```

```rust
LaunchConfig::new().with_plugin(
    freya::metrics::MetricsPlugin::default().with_visible_performance(true),
)
```

The plugin is otherwise auto-added in debug builds.

Component tree inspector: enable the `devtools` feature and run the devtools app.

## Symptom → cause → fix

| Symptom | Likely cause | Fix |
|---|---|---|
| Build fails in `skia-safe`/bindgen with "Unable to find libclang" | `LIBCLANG_PATH` unset or pointing at the wrong dir | point it at the dir containing `libclang`, e.g. `.../LLVM/bin` |
| Build fails in CMake/`ninja` during Skia | no C++ toolchain / missing `cmake` | install VS C++ workload or `cmake`+`build-essential` |
| First build takes many minutes | Skia compiling; expected | wait; keep working |
| Link error mentioning `-fuse-ld=mold` | `mold` unsupported | remove linker override |
| Compile errors on `windows-gnu` | unsupported toolchain combination | switch to `x86_64-pc-windows-msvc` |
| Window opens but is blank/black | GPU init failed, or UI has zero size | try `FREYA_RENDERER=software`; then check element sizes/`Content::Flex` |
| Window crashes on start | transparency unsupported, or bad renderer | disable `with_transparency`; force Software |
| Text invisible | color equals background, or no font | set `.color(...)` / `.theme_color()`; register a font |
| Current-thread panic "already borrowed"/deadlock | holding `read()` guard while writing same state | copy value out (`let n = *s.read();`) or scope the guard |
| Nothing re-renders after state change | state mutated via `peek`/`write_silently`, or read via `peek` only | use `read()` to subscribe |
| Lists show wrong row state after reorder | missing `.key()` | key by stable id |
| `Size::flex` children have zero size | parent lacks `.content(Content::Flex)` | add it |
| Content overflows window | used `Size::fill()` for siblings | use `Size::flex(n)` with `Content::Flex` |
| Hot reload loses state | by design | expected; restart for structural changes |
| `axum`/`tokio` API panics "no reactor running" | no Tokio context | enter a runtime in `main` (see `animation-router-async.md`) |
| App exits immediately | `with_exit_on_close(false)` with no windows left, or `main` returned | check launch config |

## Verifying an install quickly

1. `cargo new hello && cd hello`, add `freya = "0.5"`.
2. Paste the `main`/`app` from `examples/minimal/`.
3. `cargo run` — a window with centered text means toolchain, linker, Skia, windowing, fonts and
   rendering all work. Only then debug layout.

If step 3 fails, re-read the symptom table above; do not rewrite the UI, because the UI is not
under test yet.
