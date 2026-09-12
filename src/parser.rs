use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use freya_core::prelude::{
    Border, Color, ConicGradient, CornerRadius, CursorIcon, Fill, FontSlant, GradientStop,
    LinearGradient, OriginValue, Overflow, RadialGradient, Scale, Shadow, TextAlign,
    TextDecoration, TextOverflow, TransformOrigin,
};
use torin::prelude::{Alignment, Direction, Gaps, Position, Size};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CssProp {
    Width(Size),
    Height(Size),
    MinWidth(Size),
    MinHeight(Size),
    MaxWidth(Size),
    MaxHeight(Size),
    Padding(Gaps),
    Margin(Gaps),
    Background(Fill),
    CornerRadius(CornerRadius),
    Border(Option<Border>),
    BoxShadow(Shadow),
    TextColor(Fill),
    FontSize(f32),
    FontWeight(i32),
    FontFamily(String),
    TextAlign(TextAlign),
    TextDecoration(TextDecoration),
    TextOverflow(TextOverflow),
    FontSlant(FontSlant),
    Opacity(f32),
    Rotation(f32),
    Blur(f32),
    Scale(Scale),
    TransformOrigin(TransformOrigin),
    Gap(f32),
    Direction(Direction),
    MainAlign(Alignment),
    CrossAlign(Alignment),
    Hidden,
    Overflow(Overflow),
    Cursor(CursorIcon),
    Position(Position),
}

pub(crate) fn parse_inline(css: &str) -> Vec<CssProp> {
    let css = strip_comments(css);
    let css = resolve_vars(&css);
    let mut props = Vec::new();
    let mut position_mode: Option<PositionKind> = None;
    let mut position_offsets = PositionOffsets::default();

    for decl in css.split(';') {
        let decl = decl.trim();
        if decl.is_empty() {
            continue;
        }
        match decl.find(':') {
            Some(colon) => {
                let name = decl[..colon].trim().to_lowercase();
                let value = decl[colon + 1..].trim();
                match name.as_str() {
                    "position" => match parse_position_mode(value) {
                        Some(ParsedPositionMode::Mode(mode)) => position_mode = Some(mode),
                        Some(ParsedPositionMode::Static) => position_mode = None,
                        None => warn_unparsed(&format!("could not parse `position: {value}`")),
                    },
                    "top" => match parse_length_px(value) {
                        Some(v) => position_offsets.top = Some(v),
                        None => warn_unparsed(&format!("could not parse `top: {value}`")),
                    },
                    "right" => match parse_length_px(value) {
                        Some(v) => position_offsets.right = Some(v),
                        None => warn_unparsed(&format!("could not parse `right: {value}`")),
                    },
                    "bottom" => match parse_length_px(value) {
                        Some(v) => position_offsets.bottom = Some(v),
                        None => warn_unparsed(&format!("could not parse `bottom: {value}`")),
                    },
                    "left" => match parse_length_px(value) {
                        Some(v) => position_offsets.left = Some(v),
                        None => warn_unparsed(&format!("could not parse `left: {value}`")),
                    },
                    _ => match parse_declaration(&name, value) {
                        Some(prop) => props.push(prop),
                        None => warn_unparsed(&format!("could not parse `{name}: {value}`")),
                    },
                }
            }
            None => warn_unparsed(&format!("malformed declaration `{decl}` (missing ':')")),
        }
    }

    // `top`/`right`/`bottom`/`left` only take effect once `position` is `absolute`/`fixed`,
    // matching real CSS: under the default `static` flow they're silently inapplicable, not
    // a parse error, so no `CssProp::Position` is emitted without an explicit mode.
    if let Some(mode) = position_mode {
        props.push(CssProp::Position(position_offsets.build(mode)));
    }

    props
}

/// The two positioning modes this crate supports, mapping to `torin::Position`'s
/// `Absolute` (offsets from the immediate parent) and `Global` (offsets from the window,
/// the closest match to CSS `fixed`) variants. `Position::Stacked` (real CSS `static`) needs
/// no representation here since it's the element's default when no `CssProp::Position` is applied.
#[derive(Clone, Copy)]
enum PositionKind {
    Absolute,
    Global,
}

/// Outcome of parsing the `position` property's value.
enum ParsedPositionMode {
    /// An explicit positioning mode (`absolute`/`fixed`).
    Mode(PositionKind),
    /// `static`, CSS's own default — a valid, explicit no-op.
    Static,
}

/// Parses the `position` property. `static` is distinguished from an unsupported value
/// like `relative`/`sticky` so the caller only warns on the latter.
fn parse_position_mode(value: &str) -> Option<ParsedPositionMode> {
    match value.trim() {
        "absolute" => Some(ParsedPositionMode::Mode(PositionKind::Absolute)),
        "fixed" => Some(ParsedPositionMode::Mode(PositionKind::Global)),
        "static" => Some(ParsedPositionMode::Static),
        _ => None,
    }
}

#[derive(Default)]
struct PositionOffsets {
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
    left: Option<f32>,
}

impl PositionOffsets {
    fn build(self, mode: PositionKind) -> Position {
        let mut position = match mode {
            PositionKind::Absolute => Position::new_absolute(),
            PositionKind::Global => Position::new_global(),
        };
        if let Some(top) = self.top {
            position = position.top(top);
        }
        if let Some(right) = self.right {
            position = position.right(right);
        }
        if let Some(bottom) = self.bottom {
            position = position.bottom(bottom);
        }
        if let Some(left) = self.left {
            position = position.left(left);
        }
        position
    }
}

static INLINE_CACHE: LazyLock<RwLock<HashMap<String, Vec<CssProp>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Cached wrapper around [`parse_inline`] for `.css("...")` calls, which typically run
/// on every re-render with the same literal string. The cache is keyed by the exact CSS
/// source string and grows unboundedly for distinct inputs, so it's intended for static
/// string literals (as used throughout this crate's examples), not per-frame
/// `format!()`-generated CSS with ever-changing values.
pub(crate) fn parse_inline_cached(css: &str) -> Vec<CssProp> {
    if let Ok(cache) = INLINE_CACHE.read()
        && let Some(props) = cache.get(css)
    {
        return props.clone();
    }

    let props = parse_inline(css);
    if let Ok(mut cache) = INLINE_CACHE.write() {
        cache.insert(css.to_string(), props.clone());
    }
    props
}

/// Removes `/* ... */` comments before tokenizing. CSS comments do not nest.
fn strip_comments(css: &str) -> String {
    let mut result = String::with_capacity(css.len());
    let mut chars = css.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            while let Some(c) = chars.next() {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Reports a declaration that was dropped because it could not be parsed.
/// Only active in debug builds to avoid runtime cost in release.
fn warn_unparsed(message: &str) {
    #[cfg(debug_assertions)]
    eprintln!("freya-css: {message}, ignoring");
    #[cfg(not(debug_assertions))]
    let _ = message;
}

/// A single top-level result of parsing a stylesheet: either an unconditional
/// `(class, props)` rule, or one gated behind an `@media` condition.
#[derive(Debug, Clone)]
pub(crate) enum StylesheetItem {
    Rule(String, Vec<CssProp>),
    MediaRule(MediaQuery, String, Vec<CssProp>),
}

/// A simplified `@media` condition: only `min-width`/`max-width` in `px`, optionally
/// combined with `and`. Other media features (`orientation`, `prefers-color-scheme`,
/// `aspect-ratio`, `or`/`not` logic, ...) are not supported and are ignored.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct MediaQuery {
    min_width: Option<f32>,
    max_width: Option<f32>,
}

impl MediaQuery {
    pub(crate) fn matches(&self, width: f32) -> bool {
        self.min_width.is_none_or(|min| width >= min)
            && self.max_width.is_none_or(|max| width <= max)
    }
}

fn parse_media_query(condition: &str) -> MediaQuery {
    let mut query = MediaQuery::default();
    for part in condition.split("and") {
        let part = part.trim().trim_start_matches('(').trim_end_matches(')');
        if let Some((prop, value)) = part.split_once(':') {
            let value = parse_length_px(value.trim());
            match (prop.trim(), value) {
                ("min-width", Some(v)) => query.min_width = Some(v),
                ("max-width", Some(v)) => query.max_width = Some(v),
                _ => {}
            }
        }
    }
    query
}

/// Parses a CSS stylesheet string into [`StylesheetItem`]s.
///
/// Handles `.class { ... }` blocks (ignoring the leading dot), `@media (...) { ... }`
/// blocks, `@layer name { ... }` blocks (parsed transparently — cascade-layer ordering
/// itself is not implemented, contents are just flattened), nested `&:pseudo { ... }`
/// blocks (flattened into a compound `class:pseudo` rule, same convention used by
/// `.hover_class()`/`.active_class()`/`.focus_class()`), and a `:root { --name: value; }`
/// block for registering CSS variables consumed via `var(--name)`.
pub(crate) fn parse_stylesheet(css: &str) -> Vec<StylesheetItem> {
    parse_items(&strip_comments(css), None)
}

fn parse_items(css: &str, media: Option<&MediaQuery>) -> Vec<StylesheetItem> {
    let mut items = Vec::new();
    let mut remaining = css.trim_start();

    while !remaining.is_empty() {
        let Some(brace_open) = remaining.find('{') else {
            break;
        };
        let Some(brace_close) = find_matching_brace(remaining, brace_open) else {
            break;
        };

        let header = remaining[..brace_open].trim();
        let body = &remaining[brace_open + 1..brace_close];
        remaining = remaining[brace_close + 1..].trim_start();

        if let Some(condition) = header.strip_prefix("@media") {
            let query = parse_media_query(condition.trim());
            items.extend(parse_items(body, Some(&query)));
        } else if header.starts_with("@layer") {
            // Transparent: flatten contents. Layer ordering/priority is not implemented.
            items.extend(parse_items(body, media));
        } else if header == ":root" {
            parse_root_vars(body);
        } else {
            let (own_decls, nested) = split_nested_amp_blocks(body);
            let props = parse_inline(&own_decls);

            for sel in header.split(',') {
                let sel = sel.trim().trim_start_matches('.').to_string();
                if sel.is_empty() {
                    continue;
                }
                if !props.is_empty() {
                    items.push(make_item(media, sel.clone(), props.clone()));
                }
                for (suffix, nested_props) in &nested {
                    items.push(make_item(
                        media,
                        format!("{sel}{suffix}"),
                        nested_props.clone(),
                    ));
                }
            }
        }
    }

    items
}

fn make_item(media: Option<&MediaQuery>, selector: String, props: Vec<CssProp>) -> StylesheetItem {
    match media {
        Some(query) => StylesheetItem::MediaRule(query.clone(), selector, props),
        None => StylesheetItem::Rule(selector, props),
    }
}

/// Splits out one level of nested `&:pseudo { ... }` blocks from a rule body, returning
/// the remaining plain declarations plus a `(pseudo suffix, parsed props)` pair per block.
fn split_nested_amp_blocks(body: &str) -> (String, Vec<(String, Vec<CssProp>)>) {
    let mut own = String::new();
    let mut nested = Vec::new();
    let mut rest = body;

    loop {
        let Some(amp_idx) = rest.find('&') else {
            own.push_str(rest);
            break;
        };
        own.push_str(&rest[..amp_idx]);
        let after_amp = &rest[amp_idx + 1..];

        let Some(brace_open) = after_amp.find('{') else {
            own.push('&');
            rest = after_amp;
            continue;
        };
        let Some(brace_close) = find_matching_brace(after_amp, brace_open) else {
            own.push('&');
            rest = after_amp;
            continue;
        };

        let suffix = after_amp[..brace_open].trim().to_string();
        let inner = &after_amp[brace_open + 1..brace_close];
        let props = parse_inline(inner);
        if !suffix.is_empty() && !props.is_empty() {
            nested.push((suffix, props));
        }
        rest = &after_amp[brace_close + 1..];
    }

    (own, nested)
}

/// Finds the index of the `}` matching the `{` at `open_idx`, accounting for nesting.
fn find_matching_brace(s: &str, open_idx: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (i, ch) in s[open_idx..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open_idx + i);
                }
            }
            _ => {}
        }
    }
    None
}

static VARS: LazyLock<RwLock<HashMap<String, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Registers a CSS custom property (`--name`), usable via `var(--name)` afterwards.
pub(crate) fn set_var(name: &str, value: &str) {
    if let Ok(mut vars) = VARS.write() {
        vars.insert(
            name.trim_start_matches("--").to_string(),
            value.trim().to_string(),
        );
    }
}

fn parse_root_vars(body: &str) {
    for decl in body.split(';') {
        if let Some((name, value)) = decl
            .trim()
            .strip_prefix("--")
            .and_then(|d| d.split_once(':'))
        {
            set_var(name.trim(), value.trim());
        }
    }
}

/// Resolves `var(--name)` / `var(--name, fallback)` references against variables
/// registered via [`set_var`] or a parsed `:root { --name: ...; }` block. Purely
/// textual substitution resolved once wherever the declaration is parsed — no real
/// CSS cascade/inheritance, so `:root` must be declared before whatever uses its vars.
fn resolve_vars(css: &str) -> String {
    if !css.contains("var(") {
        return css.to_string();
    }

    let vars = VARS.read().ok();
    let mut result = String::with_capacity(css.len());
    let mut rest = css;

    loop {
        let Some(idx) = rest.find("var(") else {
            result.push_str(rest);
            break;
        };
        result.push_str(&rest[..idx]);
        let after = &rest[idx + 4..];

        let Some(close) = after.find(')') else {
            result.push_str("var(");
            rest = after;
            continue;
        };
        let inner = &after[..close];
        let (name, fallback) = match inner.split_once(',') {
            Some((n, f)) => (n.trim(), Some(f.trim())),
            None => (inner.trim(), None),
        };
        let name = name.trim_start_matches("--");

        let resolved = vars
            .as_ref()
            .and_then(|v| v.get(name))
            .cloned()
            .or_else(|| fallback.map(str::to_string));
        match resolved {
            Some(value) => result.push_str(&value),
            None => warn_unparsed(&format!("unresolved var(--{name})")),
        }
        rest = &after[close + 1..];
    }

    result
}

fn parse_declaration(name: &str, value: &str) -> Option<CssProp> {
    match name {
        "width" => parse_size(value).map(CssProp::Width),
        "height" => parse_size(value).map(CssProp::Height),
        "min-width" => parse_size(value).map(CssProp::MinWidth),
        "min-height" => parse_size(value).map(CssProp::MinHeight),
        "max-width" => parse_size(value).map(CssProp::MaxWidth),
        "max-height" => parse_size(value).map(CssProp::MaxHeight),
        "padding" => parse_gaps(value).map(CssProp::Padding),
        "margin" => parse_gaps(value).map(CssProp::Margin),
        "background" | "background-color" => parse_fill(value).map(CssProp::Background),
        "border-radius" => parse_corner_radius(value).map(CssProp::CornerRadius),
        "border" => Some(CssProp::Border(parse_border(value))),
        "box-shadow" => parse_shadow(value),
        "color" => parse_fill(value).map(CssProp::TextColor),
        "font-size" => parse_length_px(value).map(CssProp::FontSize),
        "font-weight" => parse_font_weight(value).map(CssProp::FontWeight),
        "font-family" => Some(CssProp::FontFamily(
            value.trim_matches('"').trim_matches('\'').to_string(),
        )),
        "text-align" => parse_text_align(value).map(CssProp::TextAlign),
        "text-decoration" => parse_text_decoration(value).map(CssProp::TextDecoration),
        "text-overflow" => parse_text_overflow(value).map(CssProp::TextOverflow),
        "font-style" => parse_font_slant(value).map(CssProp::FontSlant),
        "opacity" => parse_opacity(value).map(CssProp::Opacity),
        "rotate" => parse_angle(value).map(CssProp::Rotation),
        "scale" => parse_scale(value).map(CssProp::Scale),
        "transform-origin" => parse_transform_origin(value).map(CssProp::TransformOrigin),
        "filter" => parse_filter(value),
        "gap" | "row-gap" | "column-gap" => parse_length_px(value).map(CssProp::Gap),
        "flex-direction" => parse_flex_direction(value).map(CssProp::Direction),
        "align-items" => parse_alignment(value).map(CssProp::CrossAlign),
        "justify-content" => parse_alignment(value).map(CssProp::MainAlign),
        "display" => parse_display(value),
        "overflow" => parse_overflow(value).map(CssProp::Overflow),
        "cursor" => parse_cursor(value).map(CssProp::Cursor),
        _ => None,
    }
}

fn parse_flex_direction(value: &str) -> Option<Direction> {
    match value.trim() {
        "row" | "row-reverse" => Some(Direction::Horizontal),
        "column" | "column-reverse" => Some(Direction::Vertical),
        _ => None,
    }
}

/// Shared alignment parser for `align-items` (cross axis) and `justify-content` (main axis).
fn parse_alignment(value: &str) -> Option<Alignment> {
    match value.trim() {
        "flex-start" | "start" | "left" | "top" => Some(Alignment::Start),
        "center" => Some(Alignment::Center),
        "flex-end" | "end" | "right" | "bottom" => Some(Alignment::End),
        "space-between" => Some(Alignment::SpaceBetween),
        "space-around" => Some(Alignment::SpaceAround),
        "space-evenly" => Some(Alignment::SpaceEvenly),
        _ => None,
    }
}

/// Only `display: none` is supported, mapped to forcing a `0x0` size.
/// Unlike real CSS, this still occupies a layout node; it does not remove
/// the element from the tree the way `display: none` does.
fn parse_display(value: &str) -> Option<CssProp> {
    match value.trim() {
        "none" => Some(CssProp::Hidden),
        _ => None,
    }
}

fn parse_overflow(value: &str) -> Option<Overflow> {
    match value.trim() {
        "visible" => Some(Overflow::None),
        "hidden" | "clip" | "scroll" | "auto" => Some(Overflow::Clip),
        _ => None,
    }
}

/// Maps CSS cursor keywords (W3C `cursor` spec) to [`CursorIcon`].
fn parse_cursor(value: &str) -> Option<CursorIcon> {
    match value.trim() {
        "auto" | "default" => Some(CursorIcon::Default),
        "context-menu" => Some(CursorIcon::ContextMenu),
        "help" => Some(CursorIcon::Help),
        "pointer" => Some(CursorIcon::Pointer),
        "progress" => Some(CursorIcon::Progress),
        "wait" => Some(CursorIcon::Wait),
        "cell" => Some(CursorIcon::Cell),
        "crosshair" => Some(CursorIcon::Crosshair),
        "text" => Some(CursorIcon::Text),
        "vertical-text" => Some(CursorIcon::VerticalText),
        "alias" => Some(CursorIcon::Alias),
        "copy" => Some(CursorIcon::Copy),
        "move" => Some(CursorIcon::Move),
        "no-drop" => Some(CursorIcon::NoDrop),
        "not-allowed" => Some(CursorIcon::NotAllowed),
        "grab" => Some(CursorIcon::Grab),
        "grabbing" => Some(CursorIcon::Grabbing),
        "e-resize" => Some(CursorIcon::EResize),
        "n-resize" => Some(CursorIcon::NResize),
        "ne-resize" => Some(CursorIcon::NeResize),
        "nw-resize" => Some(CursorIcon::NwResize),
        "s-resize" => Some(CursorIcon::SResize),
        "se-resize" => Some(CursorIcon::SeResize),
        "sw-resize" => Some(CursorIcon::SwResize),
        "w-resize" => Some(CursorIcon::WResize),
        "ew-resize" => Some(CursorIcon::EwResize),
        "ns-resize" => Some(CursorIcon::NsResize),
        "nesw-resize" => Some(CursorIcon::NeswResize),
        "nwse-resize" => Some(CursorIcon::NwseResize),
        "col-resize" => Some(CursorIcon::ColResize),
        "row-resize" => Some(CursorIcon::RowResize),
        "all-scroll" => Some(CursorIcon::AllScroll),
        "zoom-in" => Some(CursorIcon::ZoomIn),
        "zoom-out" => Some(CursorIcon::ZoomOut),
        _ => None,
    }
}

pub(crate) fn parse_fill(value: &str) -> Option<Fill> {
    let value = value.trim();

    if let Some(inner) = value
        .strip_prefix("linear-gradient(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return parse_linear_gradient(inner);
    }
    if let Some(inner) = value
        .strip_prefix("radial-gradient(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return parse_radial_gradient(inner);
    }
    if let Some(inner) = value
        .strip_prefix("conic-gradient(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return parse_conic_gradient(inner);
    }

    parse_color(value).map(Fill::from)
}

fn parse_linear_gradient(inner: &str) -> Option<Fill> {
    let mut parts = split_top_level(inner, ',');

    let mut angle = 0.0;
    if let Some(a) = parts.first().and_then(|p| parse_angle(p)) {
        angle = a;
        parts.remove(0);
    }

    let stops: Vec<GradientStop> = parts
        .iter()
        .filter_map(|s| parse_gradient_stop(s))
        .collect();
    if stops.is_empty() {
        return None;
    }
    Some(LinearGradient::new().angle(angle).stops(stops).into())
}

fn parse_radial_gradient(inner: &str) -> Option<Fill> {
    let parts = split_top_level(inner, ',');
    let stops: Vec<GradientStop> = parts
        .iter()
        .filter_map(|s| parse_gradient_stop(s))
        .collect();
    if stops.is_empty() {
        return None;
    }
    Some(RadialGradient::new().stops(stops).into())
}

fn parse_conic_gradient(inner: &str) -> Option<Fill> {
    let mut parts = split_top_level(inner, ',');
    let mut gradient = ConicGradient::new();

    while let Some(head) = parts.first() {
        if let Some(rest) = head.strip_prefix("from ") {
            match rest
                .split_once(" to ")
                .map(|(s, e)| (parse_angle(s), parse_angle(e)))
            {
                Some((Some(start), Some(end))) => {
                    gradient = gradient.angles(start, end);
                    parts.remove(0);
                    continue;
                }
                _ => break,
            }
        }
        if let Some(a) = parse_angle(head) {
            gradient = gradient.angle(a);
            parts.remove(0);
            continue;
        }
        break;
    }

    let stops: Vec<GradientStop> = parts
        .iter()
        .filter_map(|s| parse_gradient_stop(s))
        .collect();
    if stops.is_empty() {
        return None;
    }
    Some(gradient.stops(stops).into())
}

/// Parses a single `<color> <offset>%` gradient stop. A missing offset defaults to `0%`.
fn parse_gradient_stop(value: &str) -> Option<GradientStop> {
    let mut tokens = split_top_level(value, ' ');
    if tokens.is_empty() {
        return None;
    }

    let mut offset = 0.0;
    if let Some(pct) = tokens
        .last()
        .and_then(|t| t.strip_suffix('%'))
        .and_then(|p| p.trim().parse::<f32>().ok())
    {
        offset = pct;
        tokens.pop();
    }
    if tokens.is_empty() {
        return None;
    }

    let color = parse_color(&tokens.join(" "))?;
    Some(GradientStop::new(color, offset))
}

pub(crate) fn parse_color(value: &str) -> Option<Color> {
    let value = value.trim();

    if let Some(hex) = value.strip_prefix('#') {
        return match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Color::from_rgb(r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::from_rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color::from_argb(a, r, g, b))
            }
            _ => None,
        };
    }

    if value.starts_with("rgba(") || value.starts_with("rgb(") {
        let inner = value
            .trim_start_matches("rgba(")
            .trim_start_matches("rgb(")
            .trim_end_matches(')');
        // Accept both legacy `rgb(r, g, b)` and CSS4 `rgb(r g b / a)` syntax.
        let normalized = inner.replace([',', '/'], " ");
        let parts: Vec<&str> = normalized.split_whitespace().collect();
        if parts.len() >= 3 {
            let r = parts[0].trim().parse::<u8>().ok()?;
            let g = parts[1].trim().parse::<u8>().ok()?;
            let b = parts[2].trim().parse::<u8>().ok()?;
            if parts.len() >= 4 {
                let a = parse_alpha_component(parts[3])?;
                return Some(Color::from_af32rgb(a, r, g, b));
            }
            return Some(Color::from_rgb(r, g, b));
        }
        return None;
    }

    if value.starts_with("hsla(") || value.starts_with("hsl(") {
        return parse_hsl(value);
    }

    match value {
        "transparent" => Some(Color::TRANSPARENT),
        "black" => Some(Color::BLACK),
        "white" => Some(Color::WHITE),
        "red" => Some(Color::RED),
        "green" => Some(Color::GREEN),
        "blue" => Some(Color::BLUE),
        "yellow" => Some(Color::YELLOW),
        "cyan" => Some(Color::CYAN),
        "magenta" => Some(Color::MAGENTA),
        "gray" | "grey" => Some(Color::GRAY),
        "darkgray" | "darkgrey" => Some(Color::DARK_GRAY),
        "lightgray" | "lightgrey" => Some(Color::LIGHT_GRAY),
        _ => None,
    }
}

/// Parses an alpha channel component: either `0.0..=1.0` or a `0%..=100%` percentage.
fn parse_alpha_component(value: &str) -> Option<f32> {
    let v = value.trim();
    if let Some(pct) = v.strip_suffix('%') {
        pct.trim().parse::<f32>().ok().map(|n| n / 100.0)
    } else {
        v.parse::<f32>().ok()
    }
}

/// Parses `hsl(...)`/`hsla(...)`, accepting both legacy comma syntax and CSS4
/// space/slash syntax, e.g. `hsl(210, 50%, 40%)` or `hsl(210 50% 40% / 50%)`.
fn parse_hsl(value: &str) -> Option<Color> {
    let inner = value
        .trim_start_matches("hsla(")
        .trim_start_matches("hsl(")
        .trim_end_matches(')');
    let normalized = inner.replace([',', '/'], " ");
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    if tokens.len() < 3 {
        return None;
    }

    let h = parse_angle(tokens[0])?;
    let s = tokens[1].trim().trim_end_matches('%').parse::<f32>().ok()? / 100.0;
    let l = tokens[2].trim().trim_end_matches('%').parse::<f32>().ok()? / 100.0;
    let a = match tokens.get(3) {
        Some(t) => parse_alpha_component(t)?,
        None => 1.0,
    };

    let (r, g, b) = hsl_to_rgb(h, s.clamp(0.0, 1.0), l.clamp(0.0, 1.0));
    Some(Color::from_af32rgb(a, r, g, b))
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s == 0.0 {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    let h = h.rem_euclid(360.0) / 360.0;
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let to_u8 = |v: f32| (v * 255.0).round() as u8;
    (
        to_u8(hue_to_rgb(p, q, h + 1.0 / 3.0)),
        to_u8(hue_to_rgb(p, q, h)),
        to_u8(hue_to_rgb(p, q, h - 1.0 / 3.0)),
    )
}

fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    let t = if t < 0.0 {
        t + 1.0
    } else if t > 1.0 {
        t - 1.0
    } else {
        t
    };
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

fn parse_length_px(value: &str) -> Option<f32> {
    value
        .trim_end_matches("px")
        .trim_end_matches("pt")
        .trim()
        .parse::<f32>()
        .ok()
}

fn parse_size(value: &str) -> Option<Size> {
    let trimmed = value.trim();
    match trimmed {
        "auto" => Some(Size::auto()),
        "fill" => Some(Size::fill()),
        v if v.starts_with("calc(") => parse_calc(v),
        v if v.ends_with('%') => v
            .trim_end_matches('%')
            .trim()
            .parse::<f32>()
            .ok()
            .map(Size::percent),
        v => parse_length_px(v).map(Size::px),
    }
}

/// A single operand of a `calc()` expression: either a fixed pixel length or a
/// percentage of the parent's resolved size along that axis.
#[derive(Clone, Copy)]
enum CalcOperand {
    Px(f32),
    Percent(f32),
}

impl CalcOperand {
    fn resolve(&self, parent: f32) -> f32 {
        match self {
            CalcOperand::Px(v) => *v,
            CalcOperand::Percent(p) => parent * p / 100.0,
        }
    }
}

fn parse_calc_operand(value: &str) -> Option<CalcOperand> {
    let v = value.trim();
    if let Some(pct) = v.strip_suffix('%') {
        pct.trim().parse::<f32>().ok().map(CalcOperand::Percent)
    } else {
        parse_length_px(v).map(CalcOperand::Px)
    }
}

/// Splits `"<a> + <b>"` / `"<a> - <b>"` on a `+`/`-` token surrounded by whitespace,
/// matching the CSS `calc()` grammar (which requires that spacing to disambiguate
/// from a unary sign).
fn split_calc_operands(s: &str) -> Option<(&str, u8, &str)> {
    let bytes = s.as_bytes();
    for i in 1..bytes.len().saturating_sub(1) {
        let c = bytes[i];
        if (c == b'+' || c == b'-') && bytes[i - 1] == b' ' && bytes[i + 1] == b' ' {
            return Some((&s[..i], c, &s[i + 1..]));
        }
    }
    None
}

/// Parses `calc(<size> + <size>)` / `calc(<size> - <size>)` into a deferred [`Size`]
/// resolved at layout time via [`Size::func_data`]. Only a single `+`/`-` combining
/// two `px`/`%` operands is supported — no nesting, multiplication or division.
fn parse_calc(value: &str) -> Option<Size> {
    let inner = value.strip_prefix("calc(")?.strip_suffix(')')?;
    let (left, op, right) = split_calc_operands(inner)?;
    let left = parse_calc_operand(left)?;
    let right = parse_calc_operand(right)?;

    Some(Size::func_data(
        move |ctx| {
            let l = left.resolve(ctx.parent);
            let r = right.resolve(ctx.parent);
            Some(if op == b'+' { l + r } else { l - r })
        },
        &value.to_string(),
    ))
}

fn parse_gaps(value: &str) -> Option<Gaps> {
    let parts: Vec<f32> = value
        .split_whitespace()
        .filter_map(parse_length_px)
        .collect();
    match parts.as_slice() {
        [a] => Some(Gaps::new_all(*a)),
        [v, h] => Some(Gaps::new_symmetric(*v, *h)),
        [t, r, b, l] => Some(Gaps::new(*t, *r, *b, *l)),
        _ => None,
    }
}

fn parse_corner_radius(value: &str) -> Option<CornerRadius> {
    let parts: Vec<f32> = value
        .split_whitespace()
        .filter_map(parse_length_px)
        .collect();
    match parts.as_slice() {
        [a] => Some(CornerRadius::from(*a)),
        [tl, tr, br, bl] => Some(CornerRadius {
            top_left: *tl,
            top_right: *tr,
            bottom_right: *br,
            bottom_left: *bl,
            smoothing: 0.0,
        }),
        _ => None,
    }
}

fn parse_border(value: &str) -> Option<Border> {
    let value = value.trim();
    if value == "none" || value == "0" {
        return None;
    }
    let mut border = Border::new();
    for part in value.split_whitespace() {
        if let Some(w) = parse_length_px(part) {
            border = border.width(w);
        } else if let Some(c) = parse_color(part) {
            border = border.fill(c);
        }
        // Ignore style keywords (solid, dashed, dotted, etc.)
    }
    Some(border)
}

fn parse_shadow(value: &str) -> Option<CssProp> {
    let value = value.trim();
    if value == "none" {
        return None;
    }
    // Collect tokens, but keep rgba(...) intact
    let tokens = split_top_level(value, ' ');
    if tokens.len() < 2 {
        return None;
    }
    let x = parse_length_px(&tokens[0])?;
    let y = parse_length_px(&tokens[1])?;

    let mut blur = 0.0;
    let mut spread = 0.0;
    let mut color = Color::BLACK;
    let mut idx = 2;

    if idx < tokens.len()
        && let Some(v) = parse_length_px(&tokens[idx])
    {
        blur = v;
        idx += 1;
    }
    if idx < tokens.len()
        && let Some(v) = parse_length_px(&tokens[idx])
    {
        spread = v;
        idx += 1;
    }
    if idx < tokens.len()
        && let Some(c) = parse_color(&tokens[idx..].join(" "))
    {
        color = c;
    }

    Some(CssProp::BoxShadow(
        Shadow::new()
            .x(x)
            .y(y)
            .blur(blur)
            .spread(spread)
            .color(color),
    ))
}

/// Splits a value on `sep` at nesting depth 0, keeping `fn(...)` groups (e.g. `rgba(...)`) intact.
fn split_top_level(value: &str, sep: char) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for ch in value.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            c if c == sep && depth == 0 => {
                let t = current.trim().to_string();
                if !t.is_empty() {
                    tokens.push(t);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let t = current.trim().to_string();
    if !t.is_empty() {
        tokens.push(t);
    }
    tokens
}

fn parse_font_weight(value: &str) -> Option<i32> {
    match value.trim() {
        "normal" => Some(400),
        "bold" => Some(700),
        "lighter" => Some(300),
        "bolder" => Some(800),
        n => n.parse::<i32>().ok(),
    }
}

fn parse_text_align(value: &str) -> Option<TextAlign> {
    match value.trim() {
        "left" => Some(TextAlign::Left),
        "right" => Some(TextAlign::Right),
        "center" => Some(TextAlign::Center),
        "justify" => Some(TextAlign::Justify),
        "start" => Some(TextAlign::Start),
        "end" => Some(TextAlign::End),
        _ => None,
    }
}

/// Only the decoration line is modeled (`freya-core`'s `TextDecoration` has no separate
/// style/color/thickness), so any of those following the keyword are simply not supported.
fn parse_text_decoration(value: &str) -> Option<TextDecoration> {
    match value.trim() {
        "none" => Some(TextDecoration::None),
        "underline" => Some(TextDecoration::Underline),
        "overline" => Some(TextDecoration::Overline),
        "line-through" => Some(TextDecoration::LineThrough),
        _ => None,
    }
}

fn parse_text_overflow(value: &str) -> Option<TextOverflow> {
    match value.trim() {
        "clip" => Some(TextOverflow::Clip),
        "ellipsis" => Some(TextOverflow::Ellipsis),
        v => v
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .or_else(|| v.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
            .map(|custom| TextOverflow::Custom(custom.to_string())),
    }
}

fn parse_font_slant(value: &str) -> Option<FontSlant> {
    match value.trim() {
        "normal" => Some(FontSlant::Upright),
        "italic" => Some(FontSlant::Italic),
        "oblique" => Some(FontSlant::Oblique),
        _ => None,
    }
}

fn parse_opacity(value: &str) -> Option<f32> {
    parse_alpha_component(value)
}

fn parse_angle(value: &str) -> Option<f32> {
    let v = value.trim();
    if v.ends_with("deg") {
        v.trim_end_matches("deg").parse::<f32>().ok()
    } else if v.ends_with("rad") {
        v.trim_end_matches("rad")
            .parse::<f32>()
            .ok()
            .map(f32::to_degrees)
    } else {
        v.parse::<f32>().ok()
    }
}

fn parse_filter(value: &str) -> Option<CssProp> {
    let v = value.trim();
    if let Some(inner) = v.strip_prefix("blur(").and_then(|s| s.strip_suffix(')')) {
        parse_length_px(inner).map(CssProp::Blur)
    } else {
        None
    }
}

/// `scale: <n>` (uniform) or `scale: <x> <y>` — the standalone CSS `scale` property
/// (CSS Transforms Level 2), not the `transform: scale(...)` function form.
fn parse_scale(value: &str) -> Option<Scale> {
    let parts: Vec<f32> = value
        .split_whitespace()
        .filter_map(|p| p.parse::<f32>().ok())
        .collect();
    match parts.as_slice() {
        [s] => Some(Scale::from(*s)),
        [x, y] => Some(Scale::from((*x, *y))),
        _ => None,
    }
}

/// `transform-origin: <keyword>`, `<keyword> <keyword>` (either order), or one/two
/// `px`/`%` values. A single value applies to `x`; `y` defaults to `center`, matching CSS.
fn parse_transform_origin(value: &str) -> Option<TransformOrigin> {
    let value = value.trim();
    if let Some(origin) = keyword_transform_origin(value) {
        return Some(origin);
    }

    match value.split_whitespace().collect::<Vec<_>>().as_slice() {
        [x] => Some(TransformOrigin {
            x: parse_origin_value(x)?,
            y: OriginValue::Fraction(0.5),
        }),
        [x, y] => Some(TransformOrigin {
            x: parse_origin_value(x)?,
            y: parse_origin_value(y)?,
        }),
        _ => None,
    }
}

fn keyword_transform_origin(value: &str) -> Option<TransformOrigin> {
    match value {
        "center" => Some(TransformOrigin::center()),
        "top" => Some(TransformOrigin::top()),
        "bottom" => Some(TransformOrigin::bottom()),
        "left" => Some(TransformOrigin::left()),
        "right" => Some(TransformOrigin::right()),
        "top left" | "left top" => Some(TransformOrigin::top_left()),
        "top right" | "right top" => Some(TransformOrigin::top_right()),
        "bottom left" | "left bottom" => Some(TransformOrigin::bottom_left()),
        "bottom right" | "right bottom" => Some(TransformOrigin::bottom_right()),
        _ => None,
    }
}

fn parse_origin_value(value: &str) -> Option<OriginValue> {
    if let Some(pct) = value.strip_suffix('%') {
        pct.trim()
            .parse::<f32>()
            .ok()
            .map(|p| OriginValue::Fraction(p / 100.0))
    } else {
        parse_length_px(value).map(OriginValue::Pixels)
    }
}

#[cfg(test)]
mod tests;
