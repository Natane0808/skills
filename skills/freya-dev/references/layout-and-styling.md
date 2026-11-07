# Layout and styling

Freya's layout engine is Torin, re-exported through `freya::prelude`. It is flex-like, but the
names and defaults differ from CSS. Every builder below is a method on the element
(`rect()`, `label()`, `paragraph()`, `image()`), available via extension traits from the prelude.

## Sizing

`Size` constructors (all `const fn`):

| Constructor | Meaning |
|---|---|
| `Size::auto()` | size to content (`Inner`). **Default.** |
| `Size::px(200.)` | fixed pixels |
| `Size::percent(50.)` | percentage of the parent's size on that axis |
| `Size::window_percent(80.)` | percentage of the root window |
| `Size::fill()` | take all remaining space along the parent's main axis |
| `Size::fill_minimum()` | match the biggest sibling (with `Content::Fit`) |
| `Size::flex(1.)` | grow by weight, shares space between siblings |
| `Size::func(\|ctx\| Some(v))` | compute at layout time |

`.width(...)` / `.height(...)` accept `impl Into<Size>`, so `200.` and `Size::px(200.)` both work.

### `fill` vs `flex` — the distinction that matters

- **`Size::fill()`** consumes the remaining space; *later siblings may overflow*. Good for
  "this page fills the window".
- **`Size::flex(n)`** participates in a weighted split with siblings. It **requires the parent to
  set `.content(Content::Flex)`**. Weights are relative: 1 and 2 split like 2 and 4. If total
  weight is below 1, each child gets `weight × remaining` and space is left unused.

```rust
// sidebar fixed, content takes the rest
rect()
    .horizontal()
    .content(Content::Flex)
    .child(rect().width(Size::px(220.)).child("Sidebar"))
    .child(rect().width(Size::flex(1.)).child("Content"))
```

## The `Content` model

| Value | Behaviour |
|---|---|
| `Content::Normal` | stack children along the main axis (**default**) |
| `Content::Fit` | match children to the widest/tallest sibling (use with `Size::fill_minimum()`) |
| `Content::Flex` | share remaining space via `Size::flex(n)` |

## Direction, alignment, spacing

```rust
rect()
    .horizontal()                 // or .vertical(); .direction(Direction::Horizontal)
    .main_align(Alignment::SpaceBetween)
    .cross_align(Alignment::Center)
    .spacing(8.)                  // gap between children
    .center()                     // shorthand: both axes centered
    .padding(12.)
    .child("A")
    .child("B")
```

- `.direction(..)`, `.main_align(..)`, `.cross_align(..)`, `.spacing(f32)`, `.content(Content)`,
  `.center()`, `.horizontal()`, `.vertical()`, `.offset_x(f32)`, `.offset_y(f32)`.
- `Alignment::{Start (default), Center, End, SpaceBetween, SpaceEvenly, SpaceAround}`.
- `.center()` centers children on **both** axes — it is not a margin-auto trick.

## Spacing: `Gaps`

`Gaps` is used for `padding` and `margin` and accepts `impl Into<Gaps>`:

```rust
rect()
    .padding(12.)                              // all sides
    .margin((8., 16.))                         // vertical, horizontal
    .padding(Gaps::new(4., 8., 4., 8.))        // top, right, bottom, left
    .padding(Gaps::new_symmetric(8., 16.))     // vertical, horizontal
```

Tuple conversions: `f32` → all sides; `(f32, f32)` → `(vertical, horizontal)`;
`(f32, f32, f32, f32)` → `(top, right, bottom, left)`.

## Box model extras

```rust
rect()
    .min_width(Size::px(100.)).max_width(Size::percent(80.))
    .min_height(Size::px(40.)).max_height(Size::px(400.))
    .expanded()                // fill remaining space
```

`VisibleSize` is an enum, not a 4-tuple: `VisibleSize::full()` (default) or
`VisibleSize::inner_percent(50.)` to use only part of the measured size in layout.

### Absolute positioning

`Position` is an enum with three variants, each taking sides via `.top() .right() .bottom() .left()`:

```rust
rect()
    .position(Position::new_absolute().top(10.).left(10.))   // relative to the parent
    .child("Badge");

rect()
    .position(Position::new_global().bottom(16.).right(16.))  // relative to the window
    .child("Floating action");
```

- `Position::new_stacked()` — default, follows normal layout flow.
- `Position::new_absolute()` — out of flow, offset from the parent.
- `Position::new_global()` — out of flow, offset from the window.

`.offset_x(f32)` / `.offset_y(f32)` displace an element relative to its normal layout position
and keep it in the flow.

For overlays and modals, prefer `Position::new_global()` combined with `Layer::Overlay` so the
element is not clipped by an ancestor with `Overflow::Clip`.

## Colors and fills

`Fill` accepts many shapes through `impl Into<Fill>`:

```rust
.background((15, 163, 242))          // RGB
.background((15, 163, 242, 200))     // RGBA u8
.background((15, 163, 242, 0.8))     // RGB + alpha f32
.background(Color::from_rgb(15, 163, 242))
.background(Color::from_hex("#0FA3F2").unwrap())
.background(0xFF0FA3F2u32)           // ARGB
```

`Color` helpers: `Color::new(u32)`, `from_rgb`, `from_argb`, `from_af32rgb`, `from_hsv`,
`from_hex(&str) -> Option<Self>`, `.with_a(u8)`, `.with_af32(f32)`, `.a() .r() .g() .b()`,
`.luminance()`, `.is_dark() / .is_light()`, `Color::lerp(a, b, t)`. Constants include
`Color::WHITE`, `Color::BLACK`, `Color::RED`, `Color::TRANSPARENT`, etc.

### Gradients

```rust
use freya::prelude::*;

rect()
    .background(
        LinearGradient::new()
            .angle(45.)
            .stops([
                GradientStop::new((255, 0, 128), 0.),
                GradientStop::new((0, 80, 255), 1.),
            ])
    )
```

`RadialGradient::new()`, `ConicGradient::new().angle(..).angles(start, end)` also convert into
`Fill` directly.

## Borders, corners, shadows

```rust
rect()
    .border(Border::new().fill(Color::BLUE).width(2.))
    .corner_radius(CornerRadius::new_all(8.))
    .shadow((0., 4., 20., 4., (0, 0, 0, 80)))   // x, y, blur, spread, color
```

- `Border::new().fill(impl Into<Color>).width(impl Into<BorderWidth>).alignment(BorderAlignment)`.
- `CornerRadius::new(tl, tr, br, bl)`, `new_all(r)`, `new_symmetric(top, bottom)`.
- Rounded shorthands: `.rounded()`, `.rounded_sm()`, `.rounded_md()`, `.rounded_lg()`,
  `.rounded_xl()`, `.rounded_2xl()`, `.rounded_3xl()`, `.rounded_4xl()`, `.rounded_full()`,
  `.rounded_none()`, plus `.with_corner_radius(f32)`.
- `Shadow` converts from `(x, y, blur, spread, color)`, or build it:
  `.shadow(Shadow::new().inset().x(0.).y(2.).blur(8.).spread(1.).color((0, 0, 0, 90)))`.

## Text styling

Available on text-bearing elements (`label`, `paragraph`) and `rect`:

```rust
label()
    .text("Title")
    .font_size(28.)
    .font_weight(FontWeight::BOLD)
    .color((245, 245, 245))
    .font_family("Inter")
    .text_align(TextAlign::Center)
    .letter_spacing(0.5)
    .text_decoration(TextDecoration::Underline)
    .text_shadow((0., 1., 2., (0, 0, 0, 120)))
```

- `.font_size(impl Into<FontSize>)`, `.font_weight(impl Into<FontWeight>)` —
  `FontWeight::{THIN(100) .. NORMAL(400), MEDIUM(500), SEMI_BOLD(600), BOLD(700), BLACK(900)}`.
- `.font_family(impl Into<Cow<'static, str>>)` — must match a system font name or a name you
  registered with `LaunchConfig::with_font`. Families are inherited by descendants.
- `.font_slant(FontSlant)`, `.font_width(FontWidth)`, `.text_height(TextHeightBehavior)`.
- Font features: `.font_tabular()`, `.font_small_caps()`, `.font_slashed_zero()`,
  `.font_fractions()`, `.font_no_ligatures()`, or `.font_feature(FontFeature)`.
- `.text_overflow(TextOverflow)` on labels; `.max_lines(Some(2))` and
  `.line_height(Some(20.))` on `label()`/`paragraph()`.

## Effects

```rust
rect()
    .opacity(0.8)
    .rotation(15.)
    .scale(Scale::new(1.1, 1.1))
    .transform_origin(TransformOrigin::center())
    .backdrop_blur(8.)
    .cursor(CursorIcon::Pointer)
    .cursor_visible(true)
    .overflow(Overflow::Clip)
```

`.overflow(Overflow::Clip)` is what you need for "hide content outside the rounded card",
and is also implied by some scroll containers.

## Scrolling

```rust
ScrollView::new()
    .height(Size::flex(1.))
    .show_scrollbar(true)
    .children(items.read().iter().map(|i| label().key(i.id).text(i.title.clone())))
```

`ScrollView` scrolls its children. For long lists, use `VirtualScrollView` so only visible rows
render:

```rust
VirtualScrollView::new(|item: VirtualItem, data: &Vec<String>| {
    rect().key(item.index).child(data[item.index].clone()).into()
})
.length(data.read().len())
.item_size(30.)
```

`.item_size(impl Into<ItemSize>)`. `ItemSize` is either `Fixed(f32)` for uniform rows or a
closure `|index: usize| -> f32` for variable heights; both convert via `.into()`:

```rust
VirtualScrollView::new(|item: VirtualItem, data: &Vec<String>| {
    rect().key(item.index).child(data[item.index].clone()).into()
})
.length(data.read().len())
.item_size(|index: usize| if index % 2 == 0 { 30. } else { 50. })
```

Note the builder closure receives `(VirtualItem, &D)`: the data is passed by reference from
`length` + the closure's captured data type `D`, so the list data does not have to be cloned
into the closure.

## Layers (stacking)

Rendering order is controlled by `.layer(..)`:

- `Layer::Relative(i16)` (default) — relative to parent; negative goes behind siblings.
- `Layer::Overlay` — a big jump above the parent's layer; stackable.
- `Layer::OverlayLevel(u8)` — a specific absolute level, up to 16.

```rust
rect()
    .child(rect().layer(-1).child("Background"))
    .child(rect().layer(1).child("Foreground"))
    .child(rect().child("Modal").layer(Layer::Overlay))
```

Ordering *within* the same layer is not guaranteed — use distinct values when order matters.

## Custom painting

For anything beyond fills (charts, canvases), use `RenderCallback`:

```rust
rect()
    .background(RenderCallback::new(|ctx: &mut FillRenderContext| {
        // ctx.canvas, ctx.font_collection, ctx.layout_node.area
    }))
    .expanded()
```

This is how the `plot` feature draws Plotters charts into an element.

## Conditional builders

```rust
rect()
    .maybe(is_active, |el| el.child("Active"))       // bool
    .map(some_option, |el, value| el.child(value))   // Option<T>
    .maybe_child(opt_element)                        // Option<impl IntoElement>
    .with(|el| el.padding(4.))                       // always, for grouping
```

Prefer one `.maybe(...)` wrapping several conditional children over repeating
`.maybe_child(cond.then(...))` for the same predicate.

## Common layout mistakes

1. **Zero-size element.** Default size is `Inner`; an empty `rect()` with no children or explicit
   size collapses. Give it a size or content.
2. **`flex` without `Content::Flex`.** Children with `Size::flex(1.)` in a parent that did not set
   `.content(Content::Flex)` will not expand.
3. **`fill()` for siblings** inside a `Content::Normal` parent — the first child eats the space
   and later ones overflow. Use `flex`.
4. **Forgetting `.expanded()`** on the root page; then the layout only occupies content size and
   `.center()` has nothing to center within.
5. **Text invisible** because `.color()` matches `.background()`. Use `.theme_color()` when using
   the theme system.
6. **Scroll container with no bounded height** — give it `Size::flex(1.)` or an explicit height,
   otherwise there is nothing to scroll within.
