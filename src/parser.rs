use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use freya_core::prelude::{
    Border, Color, ConicGradient, CornerRadius, CursorIcon, Fill, GradientStop, LinearGradient,
    Overflow, RadialGradient, Shadow, TextAlign,
};
use torin::prelude::{Alignment, Direction, Gaps, Size};

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
    Opacity(f32),
    Rotation(f32),
    Blur(f32),
    Gap(f32),
    Direction(Direction),
    MainAlign(Alignment),
    CrossAlign(Alignment),
    Hidden,
    Overflow(Overflow),
    Cursor(CursorIcon),
}

pub(crate) fn parse_inline(css: &str) -> Vec<CssProp> {
    let css = strip_comments(css);
    let css = resolve_vars(&css);
    let mut props = Vec::new();
    for decl in css.split(';') {
        let decl = decl.trim();
        if decl.is_empty() {
            continue;
        }
        match decl.find(':') {
            Some(colon) => {
                let name = decl[..colon].trim().to_lowercase();
                let value = decl[colon + 1..].trim();
                match parse_declaration(&name, value) {
                    Some(prop) => props.push(prop),
                    None => warn_unparsed(&format!("could not parse `{name}: {value}`")),
                }
            }
            None => warn_unparsed(&format!("malformed declaration `{decl}` (missing ':')")),
        }
    }
    props
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
        "opacity" => parse_opacity(value).map(CssProp::Opacity),
        "rotate" => parse_angle(value).map(CssProp::Rotation),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_solid_color_background() {
        let fill = parse_fill("#ff0000").unwrap();
        assert_eq!(fill, Fill::from(Color::from_rgb(255, 0, 0)));
    }

    #[test]
    fn parses_linear_gradient() {
        let fill =
            parse_fill("linear-gradient(250deg, #ff6432 15%, #ff0000 50%, #ffc0cb 80%)").unwrap();
        let expected: Fill = LinearGradient::new()
            .angle(250.0)
            .stop((Color::from_rgb(255, 100, 50), 15.0))
            .stop((Color::from_rgb(255, 0, 0), 50.0))
            .stop((Color::from_rgb(255, 192, 203), 80.0))
            .into();
        assert_eq!(fill, expected);
    }

    #[test]
    fn parses_linear_gradient_without_angle() {
        let fill = parse_fill("linear-gradient(red 0%, blue 100%)").unwrap();
        let expected: Fill = LinearGradient::new()
            .angle(0.0)
            .stop((Color::RED, 0.0))
            .stop((Color::BLUE, 100.0))
            .into();
        assert_eq!(fill, expected);
    }

    #[test]
    fn parses_radial_gradient() {
        let fill = parse_fill("radial-gradient(#ff6432 15%, #ff0000 50%, #ffc0cb 80%)").unwrap();
        let expected: Fill = RadialGradient::new()
            .stop((Color::from_rgb(255, 100, 50), 15.0))
            .stop((Color::from_rgb(255, 0, 0), 50.0))
            .stop((Color::from_rgb(255, 192, 203), 80.0))
            .into();
        assert_eq!(fill, expected);
    }

    #[test]
    fn parses_conic_gradient_with_angle() {
        let fill =
            parse_fill("conic-gradient(250deg, #ff6432 15%, #ff0000 50%, #ffc0cb 80%)").unwrap();
        let expected: Fill = ConicGradient::new()
            .angle(250.0)
            .stop((Color::from_rgb(255, 100, 50), 15.0))
            .stop((Color::from_rgb(255, 0, 0), 50.0))
            .stop((Color::from_rgb(255, 192, 203), 80.0))
            .into();
        assert_eq!(fill, expected);
    }

    #[test]
    fn parses_conic_gradient_with_from_to() {
        let fill = parse_fill("conic-gradient(from 10deg to 300deg, red 0%, blue 100%)").unwrap();
        let expected: Fill = ConicGradient::new()
            .angles(10.0, 300.0)
            .stop((Color::RED, 0.0))
            .stop((Color::BLUE, 100.0))
            .into();
        assert_eq!(fill, expected);
    }

    #[test]
    fn parses_gradient_with_rgba_stops_without_breaking_on_inner_commas() {
        let fill =
            parse_fill("linear-gradient(90deg, rgba(0, 0, 0, 0.5) 0%, rgb(255, 255, 255) 100%)")
                .unwrap();
        let expected: Fill = LinearGradient::new()
            .angle(90.0)
            .stop((Color::from_af32rgb(0.5, 0, 0, 0), 0.0))
            .stop((Color::from_rgb(255, 255, 255), 100.0))
            .into();
        assert_eq!(fill, expected);
    }

    #[test]
    fn text_color_accepts_gradient() {
        let props = parse_inline("color: linear-gradient(90deg, red 0%, blue 100%);");
        assert_eq!(props.len(), 1);
        match &props[0] {
            CssProp::TextColor(fill) => {
                let expected: Fill = LinearGradient::new()
                    .angle(90.0)
                    .stop((Color::RED, 0.0))
                    .stop((Color::BLUE, 100.0))
                    .into();
                assert_eq!(*fill, expected);
            }
            other => panic!("expected TextColor, got {other:?}"),
        }
    }

    #[test]
    fn invalid_gradient_syntax_returns_none() {
        assert!(parse_fill("linear-gradient()").is_none());
        assert!(parse_fill("not-a-color-or-gradient").is_none());
    }

    // ── parse_color ──────────────────────────────────────────────────────────

    #[test]
    fn parses_hex_colors() {
        assert_eq!(parse_color("#f00"), Some(Color::from_rgb(255, 0, 0)));
        assert_eq!(parse_color("#ff0000"), Some(Color::from_rgb(255, 0, 0)));
        assert_eq!(
            parse_color("#ff000080"),
            Some(Color::from_argb(0x80, 255, 0, 0))
        );
    }

    #[test]
    fn parses_rgb_and_rgba() {
        assert_eq!(
            parse_color("rgb(10, 20, 30)"),
            Some(Color::from_rgb(10, 20, 30))
        );
        assert_eq!(
            parse_color("rgba(10, 20, 30, 0.5)"),
            Some(Color::from_af32rgb(0.5, 10, 20, 30))
        );
    }

    #[test]
    fn parses_named_colors() {
        assert_eq!(parse_color("red"), Some(Color::RED));
        assert_eq!(parse_color("transparent"), Some(Color::TRANSPARENT));
        assert_eq!(parse_color("darkgray"), Some(Color::DARK_GRAY));
        assert_eq!(parse_color("darkgrey"), Some(Color::DARK_GRAY));
    }

    #[test]
    fn rejects_invalid_colors() {
        assert_eq!(parse_color("#ff"), None);
        assert_eq!(parse_color("notacolor"), None);
        assert_eq!(parse_color("rgb(1, 2)"), None);
    }

    // ── parse_size / parse_gaps / parse_corner_radius ───────────────────────

    #[test]
    fn parses_sizes() {
        assert_eq!(parse_size("10px"), Some(Size::px(10.0)));
        assert_eq!(parse_size("50%"), Some(Size::percent(50.0)));
        assert_eq!(parse_size("auto"), Some(Size::auto()));
        assert_eq!(parse_size("fill"), Some(Size::fill()));
        assert_eq!(parse_size("notasize"), None);
    }

    #[test]
    fn parses_gaps_shorthand() {
        assert_eq!(parse_gaps("10px"), Some(Gaps::new_all(10.0)));
        assert_eq!(
            parse_gaps("10px 20px"),
            Some(Gaps::new_symmetric(10.0, 20.0))
        );
        assert_eq!(
            parse_gaps("1px 2px 3px 4px"),
            Some(Gaps::new(1.0, 2.0, 3.0, 4.0))
        );
        assert_eq!(parse_gaps("1px 2px 3px"), None);
    }

    #[test]
    fn parses_corner_radius_shorthand() {
        assert_eq!(parse_corner_radius("8px"), Some(CornerRadius::from(8.0)));
        assert_eq!(
            parse_corner_radius("1px 2px 3px 4px"),
            Some(CornerRadius {
                top_left: 1.0,
                top_right: 2.0,
                bottom_right: 3.0,
                bottom_left: 4.0,
                smoothing: 0.0,
            })
        );
        assert_eq!(parse_corner_radius("1px 2px"), None);
    }

    // ── parse_border ─────────────────────────────────────────────────────────

    #[test]
    fn parses_border_width_and_color() {
        let border = parse_border("2px solid #ff0000").unwrap();
        let expected = Border::new().width(2.0).fill(Color::from_rgb(255, 0, 0));
        assert_eq!(border, expected);
    }

    #[test]
    fn border_none_is_dropped() {
        assert_eq!(parse_border("none"), None);
        assert_eq!(parse_border("0"), None);
    }

    // ── parse_shadow ─────────────────────────────────────────────────────────

    #[test]
    fn parses_shadow_full() {
        let prop = parse_shadow("0px 4px 16px 2px rgba(0, 0, 0, 0.4)").unwrap();
        match prop {
            CssProp::BoxShadow(shadow) => {
                let expected = Shadow::new()
                    .x(0.0)
                    .y(4.0)
                    .blur(16.0)
                    .spread(2.0)
                    .color(Color::from_af32rgb(0.4, 0, 0, 0));
                assert_eq!(shadow, expected);
            }
            other => panic!("expected BoxShadow, got {other:?}"),
        }
    }

    #[test]
    fn parses_shadow_minimal() {
        let prop = parse_shadow("0px 4px").unwrap();
        match prop {
            CssProp::BoxShadow(shadow) => {
                let expected = Shadow::new()
                    .x(0.0)
                    .y(4.0)
                    .blur(0.0)
                    .spread(0.0)
                    .color(Color::BLACK);
                assert_eq!(shadow, expected);
            }
            other => panic!("expected BoxShadow, got {other:?}"),
        }
    }

    #[test]
    fn shadow_none_is_dropped() {
        assert!(parse_shadow("none").is_none());
    }

    // ── parse_stylesheet ─────────────────────────────────────────────────────

    fn rule_name(item: &StylesheetItem) -> &str {
        match item {
            StylesheetItem::Rule(name, _) => name,
            other => panic!("expected StylesheetItem::Rule, got {other:?}"),
        }
    }

    fn rule_props(item: &StylesheetItem) -> &[CssProp] {
        match item {
            StylesheetItem::Rule(_, props) => props,
            other => panic!("expected StylesheetItem::Rule, got {other:?}"),
        }
    }

    #[test]
    fn parses_stylesheet_multiple_selectors_in_one_rule() {
        let rules = parse_stylesheet(".a, .b { color: red; }");
        assert_eq!(rules.len(), 2);
        assert_eq!(rule_name(&rules[0]), "a");
        assert_eq!(rule_name(&rules[1]), "b");
    }

    #[test]
    fn parses_stylesheet_multiple_rules() {
        let rules = parse_stylesheet(".card { padding: 10px; } .title { font-size: 12px; }");
        assert_eq!(rules.len(), 2);
        assert_eq!(rule_name(&rules[0]), "card");
        assert_eq!(rule_name(&rules[1]), "title");
    }

    #[test]
    fn empty_rule_body_is_skipped() {
        let rules = parse_stylesheet(".empty { }");
        assert!(rules.is_empty());
    }

    #[test]
    fn dot_prefix_is_optional() {
        let with_dot = parse_stylesheet(".btn { color: red; }");
        let without_dot = parse_stylesheet("btn { color: red; }");
        assert_eq!(rule_name(&with_dot[0]), rule_name(&without_dot[0]));
    }

    // ── comments ─────────────────────────────────────────────────────────────

    #[test]
    fn strips_comments_in_inline_css() {
        let props = parse_inline("/* bg */ background: #ff0000; /* pad */ padding: 10px;");
        assert_eq!(props.len(), 2);
    }

    #[test]
    fn strips_comments_in_stylesheet() {
        let rules = parse_stylesheet(
            "/* card style */\n.card /* selector comment */ {\n  padding: 10px; /* inline */\n}",
        );
        assert_eq!(rules.len(), 1);
        assert_eq!(rule_name(&rules[0]), "card");
        assert_eq!(rule_props(&rules[0]).len(), 1);
    }

    // ── hsl / hsla ───────────────────────────────────────────────────────────

    #[test]
    fn parses_hsl_legacy_comma_syntax() {
        // hsl(0, 100%, 50%) is pure red.
        assert_eq!(parse_color("hsl(0, 100%, 50%)"), Some(Color::RED));
    }

    #[test]
    fn parses_hsl_css4_space_syntax_with_alpha() {
        let color = parse_color("hsl(0 100% 50% / 50%)").unwrap();
        assert_eq!(color, Color::from_af32rgb(0.5, 255, 0, 0));
    }

    #[test]
    fn parses_hsla_function_name() {
        assert_eq!(
            parse_color("hsla(0, 100%, 50%, 1)"),
            Some(Color::from_af32rgb(1.0, 255, 0, 0))
        );
    }

    #[test]
    fn parses_hsl_grayscale_when_saturation_is_zero() {
        assert_eq!(
            parse_color("hsl(0, 0%, 50%)"),
            Some(Color::from_rgb(128, 128, 128))
        );
    }

    // ── rgb()/rgba() CSS4 syntax ─────────────────────────────────────────────

    #[test]
    fn parses_rgb_css4_space_syntax() {
        assert_eq!(
            parse_color("rgb(10 20 30)"),
            Some(Color::from_rgb(10, 20, 30))
        );
    }

    #[test]
    fn parses_rgb_css4_space_syntax_with_percent_alpha() {
        assert_eq!(
            parse_color("rgb(10 20 30 / 50%)"),
            Some(Color::from_af32rgb(0.5, 10, 20, 30))
        );
    }

    #[test]
    fn parses_rgba_legacy_syntax_with_percent_alpha() {
        assert_eq!(
            parse_color("rgba(10, 20, 30, 50%)"),
            Some(Color::from_af32rgb(0.5, 10, 20, 30))
        );
    }

    // ── malformed declarations ───────────────────────────────────────────────

    #[test]
    fn unparseable_declaration_is_dropped_not_panicking() {
        let props = parse_inline("background: not-a-color; width: 10px;");
        assert_eq!(props.len(), 1);
    }

    #[test]
    fn declaration_without_colon_is_dropped_not_panicking() {
        let props = parse_inline("this is garbage; width: 10px;");
        assert_eq!(props.len(), 1);
    }

    // ── gap / flex-direction / align-items / justify-content ────────────────

    #[test]
    fn parses_gap_variants() {
        for prop_name in ["gap", "row-gap", "column-gap"] {
            let props = parse_inline(&format!("{prop_name}: 12px;"));
            match &props[0] {
                CssProp::Gap(v) => assert_eq!(*v, 12.0),
                other => panic!("expected Gap, got {other:?}"),
            }
        }
    }

    #[test]
    fn parses_flex_direction() {
        assert_eq!(parse_flex_direction("row"), Some(Direction::Horizontal));
        assert_eq!(
            parse_flex_direction("row-reverse"),
            Some(Direction::Horizontal)
        );
        assert_eq!(parse_flex_direction("column"), Some(Direction::Vertical));
        assert_eq!(
            parse_flex_direction("column-reverse"),
            Some(Direction::Vertical)
        );
        assert_eq!(parse_flex_direction("diagonal"), None);
    }

    #[test]
    fn parses_alignment_keywords() {
        assert_eq!(parse_alignment("flex-start"), Some(Alignment::Start));
        assert_eq!(parse_alignment("center"), Some(Alignment::Center));
        assert_eq!(parse_alignment("flex-end"), Some(Alignment::End));
        assert_eq!(
            parse_alignment("space-between"),
            Some(Alignment::SpaceBetween)
        );
        assert_eq!(
            parse_alignment("space-around"),
            Some(Alignment::SpaceAround)
        );
        assert_eq!(
            parse_alignment("space-evenly"),
            Some(Alignment::SpaceEvenly)
        );
        assert_eq!(parse_alignment("nonsense"), None);
    }

    #[test]
    fn align_items_maps_to_cross_align_and_justify_content_to_main_align() {
        let props = parse_inline("align-items: center; justify-content: space-between;");
        assert_eq!(props.len(), 2);
        match &props[0] {
            CssProp::CrossAlign(a) => assert_eq!(*a, Alignment::Center),
            other => panic!("expected CrossAlign, got {other:?}"),
        }
        match &props[1] {
            CssProp::MainAlign(a) => assert_eq!(*a, Alignment::SpaceBetween),
            other => panic!("expected MainAlign, got {other:?}"),
        }
    }

    // ── display / overflow ───────────────────────────────────────────────────

    #[test]
    fn display_none_maps_to_hidden() {
        let props = parse_inline("display: none;");
        assert_eq!(props.len(), 1);
        assert!(matches!(props[0], CssProp::Hidden));
    }

    #[test]
    fn display_other_values_are_unsupported() {
        assert!(parse_inline("display: flex;").is_empty());
        assert!(parse_inline("display: block;").is_empty());
    }

    #[test]
    fn parses_overflow_keywords() {
        assert_eq!(parse_overflow("visible"), Some(Overflow::None));
        assert_eq!(parse_overflow("hidden"), Some(Overflow::Clip));
        assert_eq!(parse_overflow("clip"), Some(Overflow::Clip));
        assert_eq!(parse_overflow("scroll"), Some(Overflow::Clip));
        assert_eq!(parse_overflow("auto"), Some(Overflow::Clip));
        assert_eq!(parse_overflow("nonsense"), None);
    }

    // ── cursor ───────────────────────────────────────────────────────────────

    #[test]
    fn parses_common_cursor_keywords() {
        assert_eq!(parse_cursor("pointer"), Some(CursorIcon::Pointer));
        assert_eq!(parse_cursor("default"), Some(CursorIcon::Default));
        assert_eq!(parse_cursor("auto"), Some(CursorIcon::Default));
        assert_eq!(parse_cursor("grab"), Some(CursorIcon::Grab));
        assert_eq!(parse_cursor("not-allowed"), Some(CursorIcon::NotAllowed));
        assert_eq!(parse_cursor("ew-resize"), Some(CursorIcon::EwResize));
        assert_eq!(parse_cursor("nonsense"), None);
    }

    #[test]
    fn cursor_declaration_is_parsed_via_parse_inline() {
        let props = parse_inline("cursor: pointer;");
        assert_eq!(props.len(), 1);
        match &props[0] {
            CssProp::Cursor(icon) => assert_eq!(*icon, CursorIcon::Pointer),
            other => panic!("expected Cursor, got {other:?}"),
        }
    }

    // ── parse_inline_cached ──────────────────────────────────────────────────

    #[test]
    fn cached_parse_matches_uncached_parse() {
        let css = "background: #ff0000; padding: 12px; opacity: 0.5;";
        assert_eq!(parse_inline_cached(css), parse_inline(css));
    }

    #[test]
    fn cached_parse_is_stable_across_repeated_calls() {
        let css = "width: 42px; height: 24px;";
        let first = parse_inline_cached(css);
        let second = parse_inline_cached(css);
        assert_eq!(first, second);
    }

    // ── nested braces / &-nesting / @layer ────────────────────────────────────

    fn media_rule(item: &StylesheetItem) -> (&MediaQuery, &str, &[CssProp]) {
        match item {
            StylesheetItem::MediaRule(query, name, props) => (query, name, props),
            other => panic!("expected StylesheetItem::MediaRule, got {other:?}"),
        }
    }

    #[test]
    fn nested_ampersand_pseudo_flattens_to_compound_selector() {
        let rules = parse_stylesheet(".a { color: red; &:hover { color: blue; } }");
        assert_eq!(rules.len(), 2);
        assert_eq!(rule_name(&rules[0]), "a");
        assert_eq!(
            rule_props(&rules[0]),
            [CssProp::TextColor(Color::RED.into())]
        );
        assert_eq!(rule_name(&rules[1]), "a:hover");
        assert_eq!(
            rule_props(&rules[1]),
            [CssProp::TextColor(Color::BLUE.into())]
        );
    }

    #[test]
    fn at_layer_is_transparent_and_does_not_corrupt_siblings() {
        // Regression test: nested braces used to desync brace-matching entirely,
        // producing garbage selector names and cross-attributed props.
        let css = r#"
        @layer layout {
          @layer general {
            html {
              font-family: sans-serif;
            }
            a {
              color: #0000aa;
              &:hover {
                color: blue;
              }
            }
          }
          @layer warning {
            .warning {
              display: none;
            }
            .warning > :first-child {
              margin: 0;
            }
          }
        }
        "#;
        let rules = parse_stylesheet(css);
        let names: Vec<&str> = rules.iter().map(rule_name).collect();
        assert_eq!(
            names,
            vec!["html", "a", "a:hover", "warning", "warning > :first-child"]
        );
    }

    #[test]
    fn deeply_nested_braces_do_not_corrupt_later_selectors() {
        // Only one level of `&`-nesting is flattened; a second nested level inside the
        // first is left as unparseable text (dropped, with a debug warning) and produces
        // no props for `.outer`/`.outer:hover` here — but brace matching must not desync,
        // so a later, unrelated rule in the same file must still parse correctly.
        let css = ".outer { &:hover { &:not-a-real-nested-pseudo { color: red; } } } \
                   .after { color: green; }";
        let rules = parse_stylesheet(css);
        assert_eq!(rules.len(), 1);
        assert_eq!(rule_name(&rules[0]), "after");
        assert_eq!(
            rule_props(&rules[0]),
            [CssProp::TextColor(Color::GREEN.into())]
        );
    }

    // ── :root / var() ────────────────────────────────────────────────────────

    #[test]
    fn root_block_registers_vars_used_by_later_rules() {
        let rules = parse_stylesheet(
            ":root { --root-primary-1: #ff0000; } .btn { background: var(--root-primary-1); }",
        );
        assert_eq!(rules.len(), 1);
        assert_eq!(rule_name(&rules[0]), "btn");
        assert_eq!(
            rule_props(&rules[0]),
            [CssProp::Background(Color::from_rgb(255, 0, 0).into())]
        );
    }

    #[test]
    fn var_with_fallback_used_when_undefined() {
        let props = parse_inline("color: var(--root-undefined-1, #123456);");
        assert_eq!(
            props,
            [CssProp::TextColor(Color::from_rgb(0x12, 0x34, 0x56).into())]
        );
    }

    #[test]
    fn unresolved_var_without_fallback_is_dropped_not_panicking() {
        let props = parse_inline("color: var(--root-undefined-2); width: 10px;");
        assert_eq!(props.len(), 1);
    }

    #[test]
    fn set_var_is_directly_usable() {
        set_var("root-direct-1", "#abcdef");
        let props = parse_inline("background: var(--root-direct-1);");
        assert_eq!(
            props,
            [CssProp::Background(
                Color::from_rgb(0xab, 0xcd, 0xef).into()
            )]
        );
    }

    // ── calc() ───────────────────────────────────────────────────────────────

    #[test]
    fn calc_parses_to_some_size() {
        assert!(parse_size("calc(100% - 20px)").is_some());
    }

    #[test]
    fn calc_is_deterministic_for_the_same_expression() {
        assert_eq!(
            parse_size("calc(100% - 20px)"),
            parse_size("calc(100% - 20px)")
        );
    }

    #[test]
    fn calc_differs_for_different_expressions() {
        assert_ne!(
            parse_size("calc(100% - 20px)"),
            parse_size("calc(100% - 10px)")
        );
    }

    #[test]
    fn calc_requires_whitespace_around_the_operator() {
        // Matches the real CSS grammar: `calc(100%-20px)` is invalid, ambiguous with
        // a signed operand.
        assert!(parse_size("calc(100%-20px)").is_none());
    }

    #[test]
    fn calc_with_unparseable_operand_is_none() {
        assert!(parse_size("calc(banana - 20px)").is_none());
    }

    // ── @media ───────────────────────────────────────────────────────────────

    #[test]
    fn media_query_matches_min_and_max_width() {
        let query = MediaQuery {
            min_width: Some(400.0),
            max_width: Some(800.0),
        };
        assert!(!query.matches(300.0));
        assert!(query.matches(400.0));
        assert!(query.matches(600.0));
        assert!(query.matches(800.0));
        assert!(!query.matches(900.0));
    }

    #[test]
    fn parses_media_query_min_width_block() {
        let rules = parse_stylesheet("@media (min-width: 600px) { .btn { color: red; } }");
        assert_eq!(rules.len(), 1);
        let (query, name, props) = media_rule(&rules[0]);
        assert_eq!(name, "btn");
        assert_eq!(props, [CssProp::TextColor(Color::RED.into())]);
        assert_eq!(query.min_width, Some(600.0));
        assert_eq!(query.max_width, None);
    }

    #[test]
    fn parses_media_query_with_and_combinator() {
        let rules = parse_stylesheet(
            "@media (min-width: 400px) and (max-width: 800px) { .btn { color: red; } }",
        );
        let (query, ..) = media_rule(&rules[0]);
        assert_eq!(query.min_width, Some(400.0));
        assert_eq!(query.max_width, Some(800.0));
    }

    #[test]
    fn media_block_nested_inside_layer_is_still_media_scoped() {
        let rules = parse_stylesheet(
            "@layer responsive { @media (min-width: 600px) { .btn { color: red; } } }",
        );
        assert_eq!(rules.len(), 1);
        let (query, name, _) = media_rule(&rules[0]);
        assert_eq!(name, "btn");
        assert_eq!(query.min_width, Some(600.0));
    }
}
