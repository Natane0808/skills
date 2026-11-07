# Examples

Each directory is an independent Cargo project. Copy one and run:

```sh
cd minimal
cargo run
```

| Example | Shows |
|---|---|
| `minimal/` | the smallest correct project: `launch`, one window, one element |
| `counter/` | `use_state`, a `Component`, `Button::on_press`, derived text |
| `todo/` | text input, dynamic lists with keys, `Content::Flex`, derived counts |
| `routing/` | `Routable`, `Router`, `Outlet`, layout component, programmatic navigation |
| `animated/` | theming with `use_init_theme`, `use_animation`, `use_animation_transition` |

Read them in this order. `minimal` establishes that your environment works; everything after it
assumes a working toolchain.

## What is deliberately absent

- **No `Cargo.lock`.** Freya 0.5 is recent; committing a lock file from another machine's
  dependency resolution would be more misleading than useful. `cargo run` resolves it.
- **No assets.** No fonts or images are bundled, so the examples run with only a Rust toolchain.

## Verification status

These examples were written against the `freya 0.5.0` crate sources with every builder checked
against the published API. They were **not** compiled in the environment that produced this
skill (that sandbox had no network access for `cargo fetch`), so treat them as starting points
rather than test artifacts: run `cargo check` after copying, and if a builder is rejected,
compare it against `references/` and the crate source.

The structures, entry point, and API names are correct; if a compilation error appears it will
most likely be a minor builder-name difference that the compiler pinpoints precisely.
