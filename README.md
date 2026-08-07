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
- **`StyleSheet`** — a global registry for `.class { ... }` rules, loaded once at startup.
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
```

Classes are matched against `.name { ... }` blocks; the leading dot is optional and ignored.

## Supported elements

`.css()` and `.class()` are implemented per element, with the subset of properties that actually applies to it:

| Element               | Layout | Text / font | Background, border, shadow | Effects (opacity, rotate, blur) |
| ---------------------- | :----: | :---------: | :-------------------------: | :-------------------------------: |
| `Rect`                 | ✅     | ✅           | ✅                           | ✅                                 |
| `Label`, `Paragraph`   | ✅     | ✅           | —                            | —                                  |
| `Image`                | ✅     | —            | —                            | ✅                                 |

## Supported CSS properties

| Property | Values | Maps to |
| --- | --- | --- |
| `width`, `height`, `min-width`, `min-height`, `max-width`, `max-height` | `px`, `%`, `auto`, `fill` | `Size` |
| `padding`, `margin` | 1, 2, or 4 `px` values (CSS shorthand) | `Gaps` |
| `background`, `background-color` | `#rgb`, `#rrggbb`, `#rrggbbaa`, `rgb(...)`, `rgba(...)`, named colors | `Color` |
| `border-radius` | 1 or 4 `px` values | `CornerRadius` |
| `border` | `<width> <color>`, or `none` | `Border` |
| `box-shadow` | `<x> <y> [blur] [spread] [color]`, or `none` | `Shadow` |
| `color` | same color formats as `background` | text color |
| `font-size` | `px`/`pt` | `f32` |
| `font-weight` | `normal`, `bold`, `lighter`, `bolder`, or a number | `i32` |
| `font-family` | quoted or bare family name | `String` |
| `text-align` | `left`, `right`, `center`, `justify`, `start`, `end` | `TextAlign` |
| `opacity` | `0.0–1.0` or `0–100%` | `f32` |
| `rotate` | `<deg>` or `<rad>` | `f32` |
| `filter` | `blur(<px>)` | `f32` |

Named colors: `transparent`, `black`, `white`, `red`, `green`, `blue`, `yellow`, `cyan`, `magenta`, `gray`/`grey`, `darkgray`/`darkgrey`, `lightgray`/`lightgrey`.

## Installation

Not yet published to crates.io — add it as a path or git dependency:

```toml
[dependencies]
freya-css = { path = "../freya-css" }
```

## Requirements

Built against `freya`, `freya-core`, and `torin` `0.4.1`, edition 2024.
