# Events

Events are attached to **elements** (`rect`, `label`, `paragraph`, …), not to components. A
component organizes UI; the listeners live on the elements its `render` returns.

```rust
rect()
    .width(Size::px(100.))
    .height(Size::px(100.))
    .background(Color::RED)
    .on_press(|_| println!("clicked"))
```

All handlers come from `EventHandlersExt` and take `impl Into<EventHandler<Event<D>>>`, so a
closure works directly.

## Handler catalog

This is the **complete** list, taken from the `event_handlers!` invocations in
`freya-core`'s `EventHandlersExt` implementation. All are methods on elements.

**Press** — `on_press` (left click, tap, or Enter/Space when focused), `on_all_press` (any mouse
button), `on_secondary_down` (right click), `on_focus_press`

**Mouse** — `on_mouse_down`, `on_mouse_up`, `on_mouse_move`

**Pointer** (unified mouse + touch) — `on_pointer_press`, `on_pointer_down`, `on_pointer_move`,
`on_pointer_enter`, `on_pointer_leave`, `on_pointer_over`, `on_pointer_out`

**Keyboard** (require focus) — `on_key_down`, `on_key_up`

**Wheel** — `on_wheel`

**Touch** — `on_touch_start`, `on_touch_end`, `on_touch_move`, `on_touch_cancel`

**Layout / lifecycle** — `on_sized`, `on_visible`, `on_hidden`, `on_styled`

**Files** — `on_file_drop`, `on_global_file_hover`, `on_global_file_hover_cancelled`

**IME** — `on_ime_preedit`

**Accessibility** — `on_accessibility_action`

**Global** (fire once per dispatch, anywhere) — `on_global_pointer_up`, `on_global_pointer_down`,
`on_global_pointer_move`, `on_global_key_down`, `on_global_key_up`

**Capture** (run before everything else) — `on_capture_global_pointer_up`,
`on_capture_global_pointer_move`

### Prefer `on_press`

Use `on_press` for anything clickable: it covers mouse click, touch tap, and keyboard
activation (Enter/Space when focused), so accessibility works without extra code. Use
`on_mouse_*` / `on_pointer_*` only when you need pointer specifics — drag handles, canvas
tools, hover-only affordances.

## Event data

`Event<D>` derefs to `D`, so `e.global_location` and `e.global_location()` both work.

| Type | Fields |
|---|---|
| `MouseEventData` | `global_location`, `element_location`, `button: Option<MouseButton>` |
| `PointerEventData` | pointer location; `global_location()`, `element_location()`, `button()`, `is_primary()` |
| `KeyboardEventData` | `key: Key`, `code: Code`, `modifiers: Modifiers`; `try_as_str() -> Option<&str>` |
| `WheelEventData` | `source`, `phase`, `delta_x: f64`, `delta_y: f64`, `global_location`, `element_location` |
| `TouchEventData` | `global_location`, `element_location`, `finger_id: u64`, `phase: TouchPhase`, `force` |
| `FileEventData` | `cursor: CursorPoint`, `file_paths: Vec<PathBuf>` |
| `SizedEventData` | `area`, `visible_area`, `inner_sizes` |
| `VisibleEventData` | `area` |

```rust
rect()
    .on_key_down(|e: Event<KeyboardEventData>| {
        if let Some(text) = e.try_as_str() {
            println!("typed {text}");
        }
        if e.key == Key::Enter { /* submit */ }
    })
    .on_wheel(|e: Event<WheelEventData>| {
        println!("scrolled {} / {}", e.delta_x, e.delta_y);
    })
    .on_file_drop(|e: Event<FileEventData>| {
        for path in &e.file_paths { println!("dropped {path:?}"); }
    })
```

`Key`, `Code`, `Modifiers`, `NamedKey` come from the prelude (`keyboard_types`).

## Event methods

```rust
rect().on_press(|e: Event<PressEventData>| {
    e.stop_propagation();   // do not bubble to ancestors
});
```

- **`stop_propagation()`** — the current handler finishes, but ancestors do not receive *this*
  event. Has no effect on events that do not bubble (move/enter/leave, capture, global).
- **`prevent_default()`** — cancels the follow-up events this event would otherwise trigger.
  For example `on_mouse_up` calling it suppresses the `on_pointer_press` and
  `on_global_pointer_up` that would fire afterwards.

These are different tools: `stop_propagation` is about *where this event goes*;
`prevent_default` is about *what else this event causes*.

Also useful: `e.data()` for a `&D`, `.map(f)` / `.try_map(f)` to transform the payload before
passing it to a helper.

## Propagation and ordering

Events bubble from the target up through ancestors. Pressing an inner rect with handlers on
both prints `inner` then `outer`.

Dispatch priority within a frame:

1. **Capture** events
2. **Leave** events (`pointer_leave`, `pointer_out`)
3. **Enter / over** events (`pointer_enter`, `pointer_over`)
4. Everything else

Within a priority class, events are ordered by layer and cursor position, so the topmost
element under the cursor is reached first. Capture handlers are the strongest interception
point — a `prevent_default()` in a capture handler can cancel the regular pointer events for
that tick, which is how you implement "dismiss this overlay on the next click anywhere".

Global events fire once per dispatch regardless of what is under the cursor. They do not
bubble, so `stop_propagation` is meaningless for them; a non-global `prevent_default` can still
suppress the matching global event for that dispatch.

## Click combos

`EventsCombos` turns consecutive presses into a `PressEventType`. Up to four presses count while
they occur within 500 ms of each other without the pointer moving away.

```rust
rect().on_pointer_down(|e: Event<PointerEventData>| {
    match EventsCombos::<()>::pressed(e.global_location()) {
        PressEventType::Single => println!("select word"),
        PressEventType::Double => println!("select word"),
        PressEventType::Triple => println!("select line"),
        PressEventType::Quadruple => println!("select all"),
    }
})
```

The combo state is shared app-wide and every call advances it, so call `pressed` exactly once
per press.

## Focus and keyboard accessibility

Focusable elements need a stable `AccessibilityId` and the right accessibility attributes:

```rust
#[derive(PartialEq)]
struct FocusableBox;

impl Component for FocusableBox {
    fn render(&self) -> impl IntoElement {
        let a11y_id = use_a11y();          // stable id for the component's lifetime
        let focus = use_focus(a11y_id);    // Memo<Focus>

        rect()
            .a11y_id(a11y_id)
            .a11y_focusable(true)
            .a11y_role(AccessibilityRole::Button)
            .maybe(focus() == Focus::Keyboard, |el| {
                el.border(Border::new().fill(Color::BLUE).width(2.))
            })
            .on_press(move |_| a11y_id.request_focus())
            .child("Click or Tab to me")
    }
}
```

Important distinction, because the two are easy to mix up:

- **`use_focus(a11y_id)` returns `Memo<Focus>`** — the reactive *state*, read with `focus()`.
- **Focus *actions* live on the id**, via the `AccessibilityIdExt` trait:
  `a11y_id.request_focus()`, `a11y_id.request_unfocus()`, `a11y_id.is_focused()`.

`Focus::{Not, Pointer, Keyboard}` — render a visible ring only for `Keyboard` (tab focus), which
is why `maybe` on `Focus::Keyboard` is the standard idiom. `focus().is_focused()` covers both.

Because `on_press` already fires on Enter/Space when focused, do not add a manual key handler
just to support activation. For raw key handling use `KeyboardEventExt::is_press_event(&event)`
to detect the platform activation gesture.

Note what `on_press` actually delivers: `PressEventData` is an enum with four variants —
`Accessibility(AccessibilityActionEventData)`, `Mouse(MouseEventData)`,
`Keyboard(KeyboardEventData)`, `Touch(TouchEventData)`. If you need to know *how* a press
happened, match on it; most handlers can ignore it with `|_|`.

Other a11y builders: `.a11y_role(..)`, `.a11y_alt("description")`, `.a11y_auto_focus(true)`,
`.a11y_member_of(other_id)`, `.a11y_enabled(bool)`, `.a11y_builder(|node| { /* accesskit */ })`.
The globally focused id is available as `Platform::get().focused_accessibility_id`.

## Exposing events from your components

For one action, take an `EventHandler<T>` field (`EventHandler` is `Copy`):

```rust
#[derive(PartialEq)]
struct MyButton {
    on_press: EventHandler<Event<PressEventData>>,
}

impl Component for MyButton {
    fn render(&self) -> impl IntoElement {
        rect()
            .padding(8.)
            .background(Color::BLUE)
            .on_press(self.on_press.clone())
            .child("Press me")
    }
}
```

To expose *every* element handler (so callers can write `MyButton::new().on_key_down(..)`),
store `EventHandlers` and implement `EventHandlersExt`, plus `AccessibilityExt` so press
semantics are advertised:

```rust
#[derive(PartialEq)]
struct Clickable {
    event_handlers: EventHandlers,
    accessibility: AccessibilityData,
}

impl EventHandlersExt for Clickable {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers { &mut self.event_handlers }
}

impl AccessibilityExt for Clickable {
    fn accessibility_data_mut(&mut self) -> &mut AccessibilityData { &mut self.accessibility }
}

impl Component for Clickable {
    fn render(&self) -> impl IntoElement {
        rect()
            .accessibility(self.accessibility.clone())
            .event_handlers(self.event_handlers.clone())
            .child("Press me")
    }
}
```

## Common mistakes

1. **Keyboard handler "not firing"** — the element is not focused. Give it an `a11y_id` +
   `.a11y_focusable(true)` and call `focus.request_focus()`, or use `on_global_key_down`.
2. **Click handled twice** — an ancestor also has `on_press` (bubbling is working correctly).
   Add `stop_propagation()` in the inner handler.
3. **Hover handlers fire unexpectedly during a drag** — a drag-handling element should call
   `prevent_default()` on its `on_mouse_up`/`on_pointer_*` handlers.
4. **Global handler runs for clicks elsewhere** — expected; filter with
   `e.global_location().to_tuple() != (-1., -1.)` when you only want in-window events.
