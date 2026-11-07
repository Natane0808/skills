# Hooks and state

Freya 0.5's reactive core is its own (it stopped using Dioxus core at 0.4). The API resembles
Dioxus signals but differs in names and in some semantics.

## Rules of hooks

Hooks attach state to a component's scope. They work only if **the same hooks run in the same
order on every render**:

1. Call hooks only at the top level of `render` (or of a root function component).
2. Never call a hook inside a conditional.
3. Never call a hook inside a loop.
4. Never call a hook inside an event handler or async block.
5. Never call a hook outside a component.

```rust
// WRONG: hook behind a condition
if self.show {
    let s = use_state(|| 0);           // hooks shift position between renders
}

// RIGHT: always call, then use conditionally
let s = use_state(|| 0);
if self.show { /* use s */ }
```

Event handlers should capture hook values, not call hooks:

```rust
let mut open = use_state(|| false);
let on_press = move |_| *open.write() = true;   // capture
rect().on_press(on_press)
```

In async code, use the non-hook counterparts: `consume_context::<T>()` instead of
`use_consume::<T>()`, and capture `State` handles before spawning.

## `use_state`

```rust
let mut count = use_state(|| 0);
```

`State<T>` is `Copy` (no `.clone()` needed) and offers:

| Method | Purpose |
|---|---|
| `.read()` | read **and subscribe** to changes |
| `.peek()` | read **without** subscribing |
| `.write()` | mutate; notifies subscribers |
| `.set(v)` | replace value; notifies |
| `.set_if_modified(v)` | replace only if `T: PartialEq` and differs (avoids waking subscribers) |
| `.set_if_modified_and_then(v, f)` | same, then run `f` |
| `.with_mut(\|w\| ..)` | mutate through a closure |
| `.toggled()` / `.toggle()` | boolean-like toggle, requires `T: Not<Output = T> + Clone` |
| `.take()` | on `State<Option<T>>`, take and leave `None` |
| `.write_unchecked()` | non-notifying write (advanced) |

`State<T>` also derefs to `Fn() -> T`, so `count()` clones the current value — useful when you
want an owned `Copy`-ish value inside a closure.

Also available: `State::create(v)`, `State::create_in_scope(v, scope)`, `State::create_global(v)`.

### Guard hygiene

`read()` returns a guard that releases when it drops. Two patterns to avoid:

```rust
// DEADLOCK: holding a read guard while writing the same state
let r = state.read();
state.set(*r + 1);            // panics / deadlocks

// Fixed: copy out first
let n = *state.read();
state.set(n + 1);
```

Prefer `let v = *state.read();` over keeping guards alive, especially across `.await` points
and nested writes.

### Reading vs subscribing

- `.read()` subscribes: the component re-renders when the state changes.
- `.peek()` does **not** subscribe: the component will not re-render on change.

Using only `peek()` where you wanted updates is a classic "nothing happens" bug. Use `peek()`
deliberately inside effects or where you want to avoid a subscription loop.

## `use_memo` and derived values

Derive simple values directly in `render`; there is no need for a hook:

```rust
let total = items.read().len();
```

Use `use_memo` only for expensive derivations. It subscribes to any `State` read inside the
closure and recomputes only when those change:

```rust
let sorted = use_memo(move || {
    let mut v = items.read().clone();   // subscribed
    v.sort_by_key(|i| i.id);
    v
});
let view = sorted.read();
```

## Side effects

```rust
use_side_effect(move || {
    let n = *count.read();          // subscribed: reruns when count changes
    println!("count now {n}");
});
```

- `use_side_effect(f)` — reruns when any state read inside `f` changes.
- `use_after_side_effect(f)` — runs once after mount rather than immediately.
- `use_side_effect_value(f)` — returns a `State<T>` produced by the effect.
- `use_side_effect_with_deps(deps, f)` — reruns when `deps` change.
- `use_drop(f)` — cleanup on unmount.
- `use_id::<T>()` — a stable id for the current hook position (useful for a11y ids or keys).

**Do not** use an effect to copy one state into another. Derive the value, or use `use_memo`.
Effect-based syncing causes render loops and one-frame lag.

## Context

Context passes a value down the tree by **type**, without threading props. It is preferred over
`static`, `thread_local!`, or singletons because it is scoped to the subtree.

```rust
// provider (root)
fn app() -> impl IntoElement {
    use_provide_context(|| AppConfig { api_url: "https://...".into() });
    rect().child(DeepChild)
}

// consumer
#[derive(PartialEq)]
struct DeepChild;

impl Component for DeepChild {
    fn render(&self) -> impl IntoElement {
        let config = use_consume::<AppConfig>();   // panics if absent
        label().text(config.api_url.clone())
    }
}
```

- `use_provide_context(f)`, `use_provide_root_context(f)`.
- `use_consume::<T>()` (panics when missing), `use_try_consume::<T>()` (returns `Option`).
- Non-hook versions for async/non-render code: `provide_context`, `try_consume_context`,
  `consume_context`, `consume_root_context`.
- Providing the same type deeper in the tree shadows the ancestor for that subtree.

Use context for dependency injection and non-reactive values. For reactive shared state, use
Radio or `State` passed as a prop.

## Reusable components: `Readable`, `Writable`, `WritableSetter`

Never make a reusable component depend on `State<T>` if callers might source the value from a
radio, a memo, or a plain value. Accept the abstract interface instead:

| Type | Read | Write |
|---|---|---|
| `Readable<T>` | `.read()` (subscribes if source is reactive), `.peek()` | — |
| `Writable<T>` | `.read()`, `.peek()` | `.write()`, `WritableUtils` helpers |
| `WritableSetter<T>` | `.read()`, `.peek()` | `.set()`, `.set_if_modified()` via callback |

Conversions: `State<T>` → `.into_readable()` / `.into_writable()` / `.into_writable_setter()`;
`Memo<T>` and `RadioSlice` → `.into_readable()`; `RadioSliceMut` → all three;
`Writable`/`WritableSetter` → `Readable` via `.into()`; a plain owned value → non-reactive
`Readable` via `.into()`.

```rust
#[derive(PartialEq)]
struct NameField {
    value: WritableSetter<String>,
}

impl Component for NameField {
    fn render(&self) -> impl IntoElement {
        Input::new(self.value.clone())
    }
}
```

Built-in bindings: `Input::new(impl Into<WritableSetter<String>>)`,
`Switch::new(impl Into<WritableSetter<bool>>)`.

Route edits through a reducer with `WritableSetter::new(readable, callback)`:

```rust
Input::new(WritableSetter::new(
    radio.slice_current(|s| &s.name),
    move |name| { radio.apply(AppAction::SetName(name)); },
))
```

## Global state: Freya Radio

Enable with `features = ["radio"]`. Radio gives fine-grained updates: subscribers of a channel
re-render only when that channel is notified. Channels control **notification**, not access —
any radio can read and write the whole value.

```rust
use freya::prelude::*;
use freya::radio::*;

#[derive(Default, Clone)]
struct AppState {
    count: i32,
    name: String,
}

#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
enum AppChannel { Count, Name }

impl RadioChannel<AppState> for AppChannel {}

fn app() -> impl IntoElement {
    use_init_radio_station::<AppState, AppChannel>(AppState::default);
    rect().child(Counter)
}

#[derive(PartialEq)]
struct Counter;

impl Component for Counter {
    fn render(&self) -> impl IntoElement {
        let mut radio = use_radio(AppChannel::Count);
        rect()
            .child(format!("Count: {}", radio.read().count))
            .child(Button::new().on_press(move |_| radio.write().count += 1).child("+"))
    }
}
```

Methods on `Radio`: `.read()`, `.with(|r| ..)`, `.write()`, `.write_with(..)`,
`.write_channel(ch)`, `.write_silently()`, plus `slice*` variants. `RadioSlice` reads;
`RadioSliceMut` also writes (`.write()`, `.write_silently()`, `.notify()`).

### Cross-channel notification

When a write should notify more than its own channel, override `derive_channel`:

```rust
impl RadioChannel<AppState> for AppChannel {
    fn derive_channel(self, state: &AppState) -> Vec<Self> {
        match self {
            AppChannel::Count => vec![self, AppChannel::Name],
            AppChannel::Name => vec![self],
        }
    }
}
```

### Reducers

For complex transitions, implement `DataReducer` and call `.apply(action)`:

```rust
enum AppAction { Increment, SetName(String) }

impl DataReducer for AppState {
    type Channel = AppChannel;
    type Action = AppAction;

    fn reduce(&mut self, action: AppAction) -> ChannelSelection<AppChannel> {
        match action {
            AppAction::Increment => self.count += 1,
            AppAction::SetName(n) => self.name = n,
        }
        ChannelSelection::Current
    }
}

// in a component
radio.apply(AppAction::Increment);
```

`ChannelSelection::{Current, Select(ch), Silence}` — `.current()`, `.select(ch)`,
`.silence()`. Keep invariants inside the reducer, not in component effects. Async variants
exist as `DataAsyncReducer` / `RadioAsyncReducer`.

Stations can be shared across windows with `use_share_radio(|| RadioStation::create(..))`.

## Async

Freya has its own async runtime. `spawn` runs on it, so reactive state can be written directly.

| API | Lifetime | Use for |
|---|---|---|
| `spawn(fut)` | current component scope; cancelled on unmount | component-owned work |
| `spawn_in_window(fut)` | window root scope; survives the component | work that outlives its component |
| `spawn_global(fut)` | event loop; survives windows | app-level background work |

All return a `TaskHandle` (`.cancel()`, `.try_cancel()`, `.is_finished()`); `.owned()` returns an
`OwnedTaskHandle` that cancels on last drop.

```rust
let mut data = use_state(|| None);

spawn(async move {
    let result = fetch().await;
    data.set(Some(result));
});
```

Note that a bare `spawn(...)` in `render` is correct and reruns as effects do — you do **not**
need to wrap it in `use_hook(|| spawn(...))`. Some upstream examples do wrap it, which is
harmless but redundant.

### `use_future`

```rust
let task = use_future(|| async { fetch_user(42).await });

match &*task.state() {
    FutureState::Pending | FutureState::Loading => label().text("Loading..."),
    FutureState::Fulfilled(user) => label().text(user.name.clone()),
}
```

`FutureState<D>` has exactly three variants: `Pending`, `Loading`, `Fulfilled(D)`. There is no
error variant — model failure in `D`, for example `Result<T, E>`. Helpers: `.ok() -> Option<&D>`,
`.unwrap()`, `.is_loading()`, `.is_pending()`. Control with `task.start()` and `task.cancel()`.

The closure is reactive: state read **outside** the `async` block resubscribes and restarts the
future; reads inside the `async` block do not subscribe.

```rust
let user_id = use_state(|| 1);
let user = use_future(move || {
    let id = user_id();            // read outside async: subscribed
    async move { load_user(id).await }
});
```

### Timers and threads

```rust
use freya::prelude::{spawn, thread, timer};

// timer: async sleep on Freya's runtime
timer(Duration::from_millis(250)).await;

// thread: run a blocking closure off the caller's thread, awaiting its result
let value = thread(move || expensive_blocking_work()).await;
```

`timer(duration)` is an async sleep (backed by `async_io`), so it is safe to `.await` inside
`spawn`/`use_future` without blocking the UI.

`thread(closure)` runs a blocking `FnOnce` on a worker thread and returns a future; this is the
right tool for CPU-bound or synchronous I/O work. Do **not** perform blocking work directly
inside `spawn`, because that stalls the event loop and freezes the UI.

## Picking a state tool

- Local, one component → `use_state`.
- Derived, cheap → compute in `render`. Expensive → `use_memo`.
- Shared, app-wide, needs surgical updates or multi-window → Radio.
- Reusable component API → `Readable`/`Writable`/`WritableSetter`.
- Dependency injection / non-reactive shared value → context.
- Server data with caching and refetch → `freya-query` (`features = ["query"]`).

## Anti-patterns

| Anti-pattern | Why it hurts | Do instead |
|---|---|---|
| Effect copying state A into state B | render loops, one-frame lag | derive B, or `use_memo` |
| `peek()` where updates are expected | no re-render | `read()` |
| `.set(...)` with an unchanged value in a hot path | wakes subscribers needlessly | `set_if_modified` |
| Holding a read guard across a write of the same state | deadlock/panic | copy value out first |
| `Static`/`thread_local!` for shared app state | untracked, leaks between trees | context or Radio |
| Hooks inside `if`/`for`/closures | scope corruption | hoist them |

---

For theming (which is how Freya colors built-in components) see `components.md`; for
animation, routing and Tokio interop see `animation-router-async.md`.
