# freya-css

CSS-flavored styling for [Freya](https://docs.rs/freya) elements. Write real CSS syntax — hex colors, `px`/`%`, shorthand `padding`/`margin`, `box-shadow`, `border-radius` — instead of chaining builder calls by hand.

```rust
rect()
    .css("background: #1e1e2e; border-radius: 12px; padding: 20px; box-shadow: 0 4px 16px rgba(0,0,0,0.4);")
    .child(label().css("color: #cdd6f4; font-size: 22px; font-weight: bold;").text("freya-css"))
```

## Features

- **`.css("...")`** — apply inline CSS declarations directly to an element.
- **`.class("...")`** — apply one or more registered stylesheet classes (space-separated, like HTML `class`).
- **`.hover_class(name, state)`** / **`.active_class(name, state)`** / **`.focus_class(name, id)`** — like `.class()`, plus the matching `:hover`/`:active`/`:focus` ruleset while it applies.
- **`StyleSheet`** — a global registry for `.class { ... }` rules, loaded once at startup. Supports nested `&:pseudo { ... }` rules, `@layer`, `@media (min-width/max-width)`, and CSS variables via `:root { --name: ...; }` / `var(--name)`.
- **`calc()`** for `width`/`height`/`min-*`/`max-*` (a single `+`/`-` combining `px`/`%`), resolved at layout time.
- Zero runtime dependencies beyond `freya-core` and `torin` — it's a thin parsing + mapping layer, not a CSS engine.

## Quick example

```rust
use freya::prelude::*;
use freya_css::{CssExt, StyleSheet};

fn main() {
    StyleSheet::load(
        r#"
        .card {
            background: #1e1e2e;
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 4px 16px rgba(0,0,0,0.4);
        }

        .title {
            color: #cdd6f4;
            font-size: 22px;
            font-weight: bold;
        }
        "#,
    );

    launch(LaunchConfig::new().with_window(WindowConfig::new(app)))
}

fn app() -> impl IntoElement {
    rect()
        .class("card")
        .child(label().class("title").text("freya-css"))
}
```

See [`examples/basic.rs`](examples/basic.rs) for a full app with cards, badges, and buttons.

## Loading styles

```rust
// One class at a time
StyleSheet::register("btn", "background: #0066ff; border-radius: 6px; padding: 8px 16px;");

// A whole stylesheet string — pairs well with include_str!
StyleSheet::load(include_str!("styles.css"));

// ...or straight from disk at runtime
StyleSheet::load_file("styles.css")?;

// Remove one class, or everything (e.g. before loading a different theme)
StyleSheet::unregister("btn");
StyleSheet::clear();
```

Classes are matched against `.name { ... }` blocks; the leading dot is optional and ignored.

`.css("...")` calls are cached by the exact source string, so calling it with the same literal on every render (the common case) only parses once. The cache never evicts, so avoid feeding it distinct `format!()`-generated strings on every frame — prefer `.class()` with a `StyleSheet` for values that change at runtime.

## Supported elements

`.css()` and `.class()` are implemented per element, with the subset of properties that actually applies to it:

| Element               | Layout | Flex layout (gap, direction, align) | Text / font | Background, border, shadow | Effects (opacity, rotate, blur, overflow) |
| ---------------------- | :----: | :----------------------------------: | :---------: | :-------------------------: | :-------------------------------------------: |
| `Rect`                 | ✅     | ✅                                    | ✅           | ✅                           | ✅                                             |
| `Label`, `Paragraph`   | ✅     | —                                     | ✅           | —                            | —                                              |
| `Image`                | ✅     | ✅                                    | —            | —                            | ✅                                             |

`display: none` is the exception: it works on every element (it just forces size to `0x0` via `width`/`height`), including `Label`/`Paragraph`.

## Supported CSS properties

| Property | Values | Maps to |
| --- | --- | --- |
| `width`, `height`, `min-width`, `min-height`, `max-width`, `max-height` | `px`, `%`, `auto`, `fill` | `Size` |
| `padding`, `margin` | 1, 2, or 4 `px` values (CSS shorthand) | `Gaps` |
| `background`, `background-color` | `#rgb`, `#rrggbb`, `#rrggbbaa`, `rgb(...)`, `rgba(...)`, `hsl(...)`, `hsla(...)`, named colors, `linear-gradient(...)`, `radial-gradient(...)`, `conic-gradient(...)` | `Fill` |
| `border-radius` | 1 or 4 `px` values | `CornerRadius` |
| `border` | `<width> <color>`, or `none` | `Border` |
| `box-shadow` | `<x> <y> [blur] [spread] [color]`, or `none` | `Shadow` |
| `color` | same color/gradient formats as `background` | `Fill` (text) |
| `font-size` | `px`/`pt` | `f32` |
| `font-weight` | `normal`, `bold`, `lighter`, `bolder`, or a number | `i32` |
| `font-family` | quoted or bare family name | `String` |
| `text-align` | `left`, `right`, `center`, `justify`, `start`, `end` | `TextAlign` |
| `opacity` | `0.0–1.0` or `0–100%` | `f32` |
| `rotate` | `<deg>` or `<rad>` | `f32` |
| `filter` | `blur(<px>)` | `f32` |
| `gap`, `row-gap`, `column-gap` | `px` (all three map to the same single-axis gap) | `f32` (spacing) |
| `flex-direction` | `row`/`row-reverse` → horizontal, `column`/`column-reverse` → vertical | `Direction` |
| `align-items` | `flex-start`/`start`, `center`, `flex-end`/`end` | `Alignment` (cross axis) |
| `justify-content` | `flex-start`/`start`, `center`, `flex-end`/`end`, `space-between`, `space-around`, `space-evenly` | `Alignment` (main axis) |
| `display` | only `none` is supported | forces `width`/`height` to `0` |
| `overflow` | `visible` → shown, `hidden`/`clip`/`scroll`/`auto` → clipped | `Overflow` |
| `cursor` | standard [CSS cursor keywords](https://www.w3.org/TR/css-ui-3/#cursor) (`pointer`, `grab`, `not-allowed`, `ew-resize`, ...) | `CursorIcon`, via `on_pointer_enter`/`on_pointer_leave` |

Named colors: `transparent`, `black`, `white`, `red`, `green`, `blue`, `yellow`, `cyan`, `magenta`, `gray`/`grey`, `darkgray`/`darkgrey`, `lightgray`/`lightgrey`.

`rgb()`, `rgba()`, `hsl()` and `hsla()` accept both legacy comma syntax (`rgb(255, 0, 0)`) and CSS4 space/slash syntax (`rgb(255 0 0 / 50%)`); alpha accepts `0.0–1.0` or a percentage in either form.

Unparseable declarations are dropped and, in debug builds, logged to stderr instead of failing silently.

### Known layout limitations

- `gap`/`row-gap`/`column-gap` all map to the same value: Freya only has a single spacing value along the main axis, so there's no separate row/column gap.
- `display: none` forces the element's `width`/`height` to `0`, unlike real CSS it does not remove the element from the layout tree — put it last in the declaration list so no later `width`/`height` overrides it.
- `flex-direction: row-reverse`/`column-reverse` are treated the same as `row`/`column`; Freya has no reverse-order concept.
- `cursor` wires `on_pointer_enter`/`on_pointer_leave` internally to call `Cursor::set(...)` — see [Pseudo-class event conflicts](#pseudo-class-event-conflicts) for what this doesn't compose with.

### Gradients

`background` and `color` also accept gradients, mapped to Freya's `LinearGradient`, `RadialGradient` and `ConicGradient`:

```rust
rect().css("background: linear-gradient(135deg, #1e1e2e 0%, #313244 100%);")

label()
    .css("color: linear-gradient(90deg, #f38ba8 0%, #cba6f7 100%);")
    .text("Gradient text")
```

Syntax per gradient type:

| Gradient | Syntax |
| --- | --- |
| `linear-gradient` | `linear-gradient([<deg>,] <color> [<offset>%], ...)` — angle defaults to `0deg` if omitted |
| `radial-gradient` | `radial-gradient(<color> [<offset>%], ...)` |
| `conic-gradient` | `conic-gradient([<deg>,] [from <deg> to <deg>,] <color> [<offset>%], ...)` |

A stop's `<offset>%` is optional and defaults to `0%` if omitted. CSS side/corner keywords (`to bottom right`) and radial shape/position keywords (`circle at center`) are not supported — use an explicit angle instead.

### Hover support

`.hover_class(name, hovered)` applies `name`, and additionally applies `<name>:hover` while the pointer is over the element. Register the hover ruleset with a `:hover` suffix, exactly like any other class:

```rust
use freya::prelude::*;
use freya_css::{CssExt, HoverExt, StyleSheet};

StyleSheet::load(
    ".btn { background: #333333; }
     .btn:hover { background: #555555; }",
);

fn button() -> impl IntoElement {
    let hovered = use_state(|| false);
    rect().hover_class("btn", hovered)
}
```

The hover flag is a normal `State<bool>` (created with `use_state`, like any other Freya state), so it composes with Freya's own reactivity — no extra plumbing needed, and it works for `Rect`, `Label`, `Paragraph` and `Image` alike.

### Active support

`.active_class(name, active)` works the same way as `.hover_class()`, but applies `<name>:active` while the element is pressed:

```rust
StyleSheet::load(
    ".btn { background: #333333; }
     .btn:active { background: #111111; }",
);

fn button() -> impl IntoElement {
    let active = use_state(|| false);
    rect().active_class("btn", active)
}
```

Mouse-only: Freya has no unified pointer "release" event covering touch, so this uses `on_mouse_down`/`on_mouse_up`, plus `on_pointer_leave` to reset the active state if the button is released after dragging off the element.

### Focus support

`.focus_class(name, a11y_id)` applies `<name>:focus` while the element holds accessibility focus. Unlike hover/active, it doesn't grant focus itself — wire that up separately (e.g. on click, or rely on keyboard navigation), same as real CSS `:focus` never grants focus on its own:

```rust
StyleSheet::load(
    ".input { border: 1px #333333; }
     .input:focus { border: 1px #89b4fa; }",
);

fn input() -> impl IntoElement {
    let a11y_id = use_a11y();
    rect()
        .focus_class("input", a11y_id)
        .on_mouse_down(move |_| a11y_id.request_focus())
}
```

Since `focus_class` doesn't bind any pointer/mouse event, it composes freely with `hover_class`, `active_class` and `cursor` on the same element.

### Pseudo-class event conflicts

Freya stores a single handler per event name, so combining these on the *same* element can silently override one of them — whichever is applied last in the builder chain wins for the shared event:

| Combining... | ...conflicts on |
| --- | --- |
| `cursor` + `.hover_class()` | `on_pointer_enter`, `on_pointer_leave` |
| `cursor` + `.active_class()` | `on_pointer_leave` |
| `.hover_class()` + `.active_class()` | `on_pointer_leave` |
| `.focus_class()` + anything | no conflict — binds no pointer/mouse event |

Put `cursor` inside a class's own ruleset rather than mixing it with `.hover_class()`/`.active_class()` on the same element.

### Nesting and `@layer`

`StyleSheet::load()`/`register()`/`load_file()` understand one level of CSS nesting and `@layer`:

```rust
StyleSheet::load(
    "@layer components {
        a {
            color: #0000aa;
            &:hover {
                color: blue;
            }
        }
    }",
);
```

`&:pseudo { ... }` is flattened into a compound `selector:pseudo` rule — the exact same mechanism `.hover_class()`/`.active_class()`/`.focus_class()` already look up, so nested `&:hover`/`&:active`/`&:focus` blocks work with them for free. `@layer name { ... }` is parsed transparently: its contents are flattened into the registry as if unwrapped. Real cascade-layer ordering/priority between layers is **not** implemented — this crate has no specificity model to layer on top of in the first place.

### CSS variables

```rust
StyleSheet::load(
    ":root { --primary: #89b4fa; }
     .btn { background: var(--primary); }
     .accent { background: var(--undefined, #ff0000); }", // fallback used if unset
);

// or set one programmatically:
StyleSheet::set_var("primary", "#89b4fa");
```

This is textual substitution, not real CSS custom properties — there's no inheritance or cascade by position in the tree (that would need mutable access to the built element tree, which `freya-core` doesn't expose to external crates). Resolution happens once, top-to-bottom, wherever a declaration using `var(...)` gets parsed, so declare `:root { ... }` before anything that reads its variables.

### `calc()`

```rust
rect().css("width: calc(100% - 40px);")
```

Only a single `+`/`-` combining two `px`/`%` operands is supported (matches the vast majority of real-world usage), applied to `width`/`height`/`min-width`/`min-height`/`max-width`/`max-height`. No nesting, multiplication, division, or mixing with other units. The whitespace around `+`/`-` is required, same as real CSS (`calc(100%-20px)` is invalid — ambiguous with a signed operand).

### `@media`

```rust
StyleSheet::load(
    ".sidebar { width: 200px; }
     @media (min-width: 900px) { .sidebar { width: 320px; } }",
);
```

Only `min-width`/`max-width` in `px`, optionally combined with `and`, are supported — no `orientation`, `prefers-color-scheme`, `aspect-ratio`, or `or`/`not` logic. Reads Freya's actual window size (`Platform::get().root_size`, reactive) so it responds to resizing live, but only for classes that actually have `@media` rules registered — plain `.class()` calls never pay for it.

## Not supported

Investigated against the `freya-core`/`torin` source directly, not just deprioritized:

- **Combinator selectors** (`.a .b`, `.a > .b`) and pseudo-elements (`::before`/`::after`): need mutable access to the already-built element tree to match by ancestor/sibling. `freya-core` 0.4.1 doesn't expose that to external crates — the only third-party hook (`FreyaPlugin`, in `freya-winit`) only passes a read-only `&Tree`, and `ElementExt` has no setters. Would need a fork of `freya-core`.
- **`!important`**: this crate has no real specificity model (just declaration order), so it would only solve a problem this crate mostly doesn't have.
- **Real CSS variable inheritance** (cascading by tree position): same tree-access limitation as combinators. The variables implemented here are global textual substitution, not scoped inheritance.

## Installation

Not yet published to crates.io — add it as a path or git dependency:

```toml
[dependencies]
freya-css = { path = "../freya-css" }
```

## Requirements

Built against `freya`, `freya-core`, and `torin` `0.4.1`, edition 2024.
