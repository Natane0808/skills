# Built-in components and theming

All of these come from `freya::prelude::*` (they are re-exported from `freya-components`).
For a live visual reference, see [freyaui.dev/demo](https://freyaui.dev/demo) — as explained at
the end of this file, the in-crate `gallery()` function is only a documentation stub.

Constructors below are verified against `freya-components 0.5.0`. Where a builder is not
listed, check the crate source rather than guessing.

## Theming first

Freya's built-in components read their colors from a `Theme`. Providing a theme is the cheapest
way to make an app look coherent, and it also fixes "invisible text on dark background" bugs,
because components then follow the palette instead of hardcoded defaults.

```rust
fn app() -> impl IntoElement {
    let mut theme = use_init_theme(light_theme);   // or dark_theme

    rect()
        .expanded()
        .theme_background()
        .theme_color()
        .padding(16.)
        .child(Button::new().on_press(move |_| theme.set(dark_theme())).child("Dark"))
}
```

- `use_init_theme(|| Theme)` — register the theme; returns `State<Theme>`, so writing to it
  switches the theme app-wide.
- `use_provide_theme(|| Theme)` — same, but always creates a new one rather than reusing an
  existing context.
- `use_theme()` — consume the current `State<Theme>`.
- `get_theme_or_default()` — consume with a fallback when no theme was provided.

Element helpers that pull from the theme: `rect().theme_background()`, `.theme_color()`;
`label().theme_color()`; `SvgViewer::new(..).theme_color()`, `.theme_accent_color()`,
`.theme_fill()`, `.theme_stroke()`, `.theme_accent_fill()`, `.theme_accent_stroke()`.

**Typography helpers** — `label()` also exposes theme-driven sizes, so text hierarchy need not
hardcode numbers: `.title()`, `.subtitle()`, `.body()`, `.caption()`, `.overline()`. These map to
the `typography` theme entry (`title`, `subtitle`, `body`, `caption`, `overline`).

### Follow the OS preference

```rust
let mut theme = use_init_theme(|| Platform::get().preferred_theme.to_theme());
```

`FromPreference::to_theme()` maps `PreferredTheme::{Light, Dark}` to a theme.

### Custom themes

Start from `light_theme()` / `dark_theme()` and override fields:

```rust
fn brand_theme() -> Theme {
    let mut theme = dark_theme();
    theme.name = "brand";
    theme.colors = ColorsSheet {
        primary: Color::from_rgb(37, 52, 63),
        secondary: Color::from_rgb(255, 155, 81),
        ..DARK_COLORS
    };
    theme
}
```

`ColorsSheet` fields (these are also the valid `Preference::Reference` keys):

- Brand: `primary`, `secondary`, `tertiary`
- Status: `success`, `warning`, `error`, `info`
- Surfaces: `background`, `surface_primary`, `surface_secondary`, `surface_tertiary`,
  `surface_inverse`, `surface_inverse_secondary`, `surface_inverse_tertiary`
- Borders: `border`, `border_focus`, `border_disabled`
- Text: `text_primary`, `text_secondary`, `text_placeholder`, `text_inverse`, `text_highlight`
- Interaction: `focus`, `active`, `disabled`
- Utility: `overlay`, `shadow`

Note there is no `hover` field; use `active` or `focus` for hover feedback.

### Theming your own components

`define_theme!` generates the three-typed theme model (partial / preference / resolved):

```rust
define_theme! {
    %[component]
    pub StatusBadge {
        %[fields]
        background: Color,
        color: Color,
        corner_radius: CornerRadius,
        padding: Gaps,
    }
}

#[derive(PartialEq)]
struct StatusBadge {
    theme: Option<StatusBadgeThemePartial>,
}

impl Component for StatusBadge {
    fn render(&self) -> impl IntoElement {
        let StatusBadgeTheme { background, color, corner_radius, padding } =
            get_theme!(&self.theme, StatusBadgeThemePreference, "status_badge");

        rect()
            .background(background)
            .corner_radius(corner_radius)
            .padding(padding)
            .child(label().text("Active").color(color).font_size(12.))
    }
}
```

Generated types: `StatusBadgeThemePartial` (per-field `Option<Preference<T>>` for instance
overrides), `StatusBadgeThemePreference` (per-field `Preference<T>`, registered on the theme),
`StatusBadgeTheme` (resolved values). The `%[component]` marker also generates a
`StatusBadgeThemePartialExt` trait giving per-field builder methods on the component.

Register the default preference:

```rust
theme.set("status_badge", StatusBadgeThemePreference {
    background: Preference::Reference("secondary"),    // resolved from ColorsSheet
    color: Preference::Reference("text_inverse"),
    corner_radius: CornerRadius::new_all(99.).into(),  // Specific
    padding: Gaps::new(4., 10., 4., 10.).into(),
});
```

**Only `Color` supports `Preference::Reference`.** Passing a `Reference` for `Size`, `Gaps`,
`CornerRadius`, `f32` or `Duration` panics — use `Specific` (via `.into()`).

`get_theme!` panics on an unregistered key; use `get_theme_or_default!(&self.theme, T, "key", || default)` when the key may be absent.

## Layout containers

| Component | Constructor | Notes |
|---|---|---|
| `Card` | `Card::new()` | `.style_variant(..)`, `.layout_variant(..)`, `.hoverable(bool)`, `.on_press(..)`, `.filled() .outline() .compact()`, `.theme_colors(..)` |
| `ScrollView` | `ScrollView::new()` | `.show_scrollbar(bool)`; needs bounded height |
| `VirtualScrollView` | `VirtualScrollView::new(\|item, &data\| ..)` | `.length(n)`, `.item_size(..)` |
| `ResizableContainer` | `ResizableContainer::new()` | with `ResizablePanel::new(PanelSize)` and `ResizableHandle::new(index)` |
| `Attached` | `Attached::new(inner_element)` | attach auxiliary elements to an inner element |
| `OverflowedContent` | `OverflowedContent::new()` | fade/indicate overflowing content |
| `Lazy` | `Lazy::new()` | defer rendering until visible |
| `Skeleton` | `Skeleton::new(loading: bool)` | loading placeholder |
| `CircularLoader` | `CircularLoader::new()` | spinner |
| `ProgressBar` | `ProgressBar::new(progress: f32)` | progress in 0..=1 |
| `SideBarItem` | `SideBarItem::new()` | sidebar entry |
| `Tile` | `Tile::new()` | draggable tile |

## Inputs and controls

| Component | Constructor | Notes |
|---|---|---|
| `Button` | `Button::new()` | `.on_press(..)`, `.enabled(bool)`, `.filled() .outline() .flat()`, `.compact() .expanded()`, `.style_variant(..)` |
| `Input` | `Input::new(value: impl Into<WritableSetter<String>>)` | `.placeholder(..)`, `.on_submit(EventHandler<String>)`, `.multiline(bool)`, `.leading(..)`, `.trailing(..)`, `.auto_focus(bool)` |
| `Switch` | `Switch::new(value: impl Into<WritableSetter<bool>>)` | **no** `.toggled()`, `.on_toggle()`, or `Default` |
| `Checkbox` | `Checkbox::new()` | `.selected(bool)`, `.size(f32)` |
| `Slider` | `Slider::new(on_moved: impl Into<EventHandler<f64>>)` | `.value(f64)`, `.step(f64)`, `.size(Size)`, `.direction(Direction)` |
| `Select` | `Select::new()` | `.selected_item(Element)`, `.list_match_width(bool)` |
| `SegmentedButton` | `SegmentedButton::new()` | children are `ButtonSegment::new().selected(bool).on_press(..)` |
| `RadioItem` | `RadioItem::new()` | `.selected(bool)` |
| `Chip` | `Chip::new()` | `.selected(bool)`, `.enabled(bool)`, `.on_press(..)` |
| `ColorPicker` | `ColorPicker::new(on_change: EventHandler<Color>)` | |
| `CalendarMonth`/`CalendarGrid` | `CalendarMonth::new(..)`, `CalendarGrid::new(..)` | feature `calendar` |

Binding state to inputs passes the state directly, since `State<T>` converts into
`WritableSetter<T>`:

```rust
let mut name = use_state(String::new);
let mut on = use_state(|| false);
let mut volume = use_state(|| 0.5f64);

rect()
    .spacing(8.)
    .child(Input::new(name).placeholder("Your name"))
    .child(Switch::new(on))
    .child(Slider::new(move |v| volume.set(v)).value(*volume.read()))
```

## Overlays and feedback

| Component | Constructor | Notes |
|---|---|---|
| `Popup` | `Popup::new()` | `.on_close_request(EventHandler<()>)`; compose with `PopupTitle::new(Readable<String>)`, `PopupContent::new()`, `PopupButtons::new()` |
| `Tooltip` | `Tooltip::new()` | wrap with `TooltipContainer::new(tooltip)` |
| `ContextMenu` | `ContextMenuViewer::new()` | |
| `Menu` | `Menu::new()` | with `MenuContainer`, `MenuItem`, `MenuButton`, `SubMenu` |
| `Portal` | `Portal::new(id)` | render into a detached layer; pairs with `PortalsMap` |
| `Accordion` | `Accordion::new()` | |
| `Attached` | `Attached::new(inner)` | anchored widgets |

## Media and text

| Component | Constructor |
|---|---|
| `ImageViewer` | `ImageViewer::new(source: impl Into<ImageSource>)`; own builders `.decode_mode(..)`, `.loading_placeholder(Element)`, `.error_renderer(..)`, `.asset_age(..)`; plus everything from `ImageExt`: `.sampling_mode(..)`, `.aspect_ratio(..)`, `.image_cover(..)`, `.blur(f32)`, `.snap_to_grid(bool)` |
| `SvgViewer` | `SvgViewer::new(source: impl Into<ImageSource>)`; `.color(..)`, `.async_rasterization(bool)`, theme helpers |
| `SelectableText` | `SelectableText::new()` |
| `MarkdownViewer` | `MarkdownViewer::new(content: impl Into<Cow<'static, str>>)`; `.inline_element(\|html: String\| Option<Element>)`, `.code_editor_font_family(..)`, plus theme field setters (`heading_h1`…`heading_h6`, `paragraph_size`, `code_font_size`, ...) — feature `markdown` |
| `CodeEditor` | `CodeEditor::new(data: CodeEditorData, a11y_id)` — feature `code-editor`; supports arbitrary languages via `EditorLanguage::new(tree_sitter_language, highlights_query)` |

`ImageViewer` implements the same layout/style traits as elements (`LayoutExt`,
`ContainerSizeExt`, `DecorationExt`, `EffectExt`, `StyleExt`, `ImageExt`), so `.width(..)`,
`.rounded()`, `.opacity(..)` etc. work on it too. Remote image sources need
`features = ["remote-asset"]`. Avoid decoding images larger than their display size with
`.decode_mode(DecodeMode::FromLayout | DecodeMode::Custom(Size2D) | DecodeMode::Source)`;
`FromLayout` is the default.

## Tables and trees

`Table::new()` with `.column_widths(vec![Size..])`, composed from `TableRow::new()` items and
configured via `TableConfig`. `Accordion` covers collapsible sections; `Lazy` covers deferred
subtrees.

## Drag and drop

```rust
DragZone::new(data)                       // data: T (Clone + PartialEq)
DropZone::new(move |data: T| { .. })      // on_drop handler
```

For file drops from the OS, use the element event `on_file_drop` instead
(`FileEventData::file_paths`).

## Routing-related components (feature `router`)

| Component | Purpose |
|---|---|
| `Router::<Route>::new(f)` | mounts the router |
| `Outlet::<Route>::new()` | renders the matched route |
| `Link::new(target)` | declarative navigation |
| `ActivableRoute::new(route, child)` | style a child when a route is active |
| `AnimatedRouter::new(child)` | animate route transitions |
| `FloatingTab::new()` | tab styled for bottom bars |

## Icons (feature `icons`)

Lucide icons are rendered as SVGs:

```rust
use freya::icons;

SvgViewer::new(icons::lucide::antenna())
    .color((120, 50, 255))
    .width(Size::px(24.))
    .height(Size::px(24.))
```

`features = ["icons-lucide"]` enables the full Lucide set (implies `icons`).

## Material Design (feature `material-design`)

Ripple support is added by extension traits, one per host component. `.ripple()` **returns a
different type** (`RippleButton`), so call it before `.child(..)`:

```rust
use freya::material_design::*;

Button::new()
    .on_press(|_| println!("pressed"))
    .ripple()                       // -> RippleButton
    .child("Click me")
```

Extension traits: `ButtonRippleExt` (`.ripple()` on `Button`), `FloatingTabRippleExt`,
`MenuItemRippleExt`, `SideBarItemRippleExt`, `TileRippleExt`. Each has a matching `Ripple*`
component (`RippleButton`, `RippleFloatingTab`, `RippleMenuItem`, `RippleSideBarItem`,
`RippleTile`) if you want to construct it directly. Import the prelude so the traits are in
scope.

## Where to see every component at once

`freya::components::gallery` is **only a documentation stub** (`pub fn gallery() {}`); calling it
renders nothing. To see the components live, use the interactive gallery at
[freyaui.dev/demo](https://freyaui.dev/demo), or browse the `examples/component_*` files in the
upstream repository.

Do not build a "show me all components" screen from `gallery()` — it is a no-op and its return
type is `()`, so it will not compile as a child element.

## Choosing between element and component

Reach for an **element** (`rect`, `label`, `paragraph`, `image`) when you need pixels and
control. Reach for a **component** when you want behaviour, theming, accessibility and focus
handling for free. Hand-rolling a button out of `rect` + `on_press` loses keyboard activation,
focus rings, disabled states and theme colors — use `Button`.

Conversely, do not fight a built-in component's styling; if you need a very different look,
compose primitives or define your own theme instead of overriding every theme field.
