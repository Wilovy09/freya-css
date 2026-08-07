use freya_core::prelude::{
    Border, Color, CornerRadius, Shadow, TextAlign,
};
use torin::prelude::{Gaps, Size};

#[derive(Debug, Clone)]
pub(crate) enum CssProp {
    Width(Size),
    Height(Size),
    MinWidth(Size),
    MinHeight(Size),
    MaxWidth(Size),
    MaxHeight(Size),
    Padding(Gaps),
    Margin(Gaps),
    Background(Color),
    CornerRadius(CornerRadius),
    Border(Option<Border>),
    BoxShadow(Shadow),
    TextColor(Color),
    FontSize(f32),
    FontWeight(i32),
    FontFamily(String),
    TextAlign(TextAlign),
    Opacity(f32),
    Rotation(f32),
    Blur(f32),
}

pub(crate) fn parse_inline(css: &str) -> Vec<CssProp> {
    let mut props = Vec::new();
    for decl in css.split(';') {
        let decl = decl.trim();
        if decl.is_empty() {
            continue;
        }
        if let Some(colon) = decl.find(':') {
            let name = decl[..colon].trim().to_lowercase();
            let value = decl[colon + 1..].trim();
            if let Some(prop) = parse_declaration(&name, value) {
                props.push(prop);
            }
        }
    }
    props
}

/// Parses a CSS stylesheet string into (class_name, props) pairs.
/// Handles `.class { ... }` blocks, ignoring the leading dot.
pub(crate) fn parse_stylesheet(css: &str) -> Vec<(String, Vec<CssProp>)> {
    let mut rules = Vec::new();
    let mut remaining = css;

    while let Some(brace_open) = remaining.find('{') {
        let selector = remaining[..brace_open].trim();
        remaining = &remaining[brace_open + 1..];

        if let Some(brace_close) = remaining.find('}') {
            let body = &remaining[..brace_close];
            remaining = &remaining[brace_close + 1..];

            let props = parse_inline(body);
            if !props.is_empty() {
                for sel in selector.split(',') {
                    let sel = sel.trim().trim_start_matches('.').to_string();
                    if !sel.is_empty() {
                        rules.push((sel, props.clone()));
                    }
                }
            }
        }
    }

    rules
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
        "background" | "background-color" => parse_color(value).map(CssProp::Background),
        "border-radius" => parse_corner_radius(value).map(CssProp::CornerRadius),
        "border" => Some(CssProp::Border(parse_border(value))),
        "box-shadow" => parse_shadow(value),
        "color" => parse_color(value).map(CssProp::TextColor),
        "font-size" => parse_length_px(value).map(CssProp::FontSize),
        "font-weight" => parse_font_weight(value).map(CssProp::FontWeight),
        "font-family" => Some(CssProp::FontFamily(
            value.trim_matches('"').trim_matches('\'').to_string(),
        )),
        "text-align" => parse_text_align(value).map(CssProp::TextAlign),
        "opacity" => parse_opacity(value).map(CssProp::Opacity),
        "rotate" => parse_angle(value).map(CssProp::Rotation),
        "filter" => parse_filter(value),
        _ => None,
    }
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
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() >= 3 {
            let r = parts[0].trim().parse::<u8>().ok()?;
            let g = parts[1].trim().parse::<u8>().ok()?;
            let b = parts[2].trim().parse::<u8>().ok()?;
            if parts.len() == 4 {
                let a = parts[3].trim().parse::<f32>().ok()?;
                return Some(Color::from_af32rgb(a, r, g, b));
            }
            return Some(Color::from_rgb(r, g, b));
        }
        return None;
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

fn parse_length_px(value: &str) -> Option<f32> {
    value
        .trim_end_matches("px")
        .trim_end_matches("pt")
        .trim()
        .parse::<f32>()
        .ok()
}

fn parse_size(value: &str) -> Option<Size> {
    match value.trim() {
        "auto" => Some(Size::auto()),
        "fill" => Some(Size::fill()),
        v if v.ends_with('%') => v
            .trim_end_matches('%')
            .trim()
            .parse::<f32>()
            .ok()
            .map(Size::percent),
        v => parse_length_px(v).map(Size::px),
    }
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
    let tokens = split_shadow_tokens(value);
    if tokens.len() < 2 {
        return None;
    }
    let x = parse_length_px(&tokens[0])?;
    let y = parse_length_px(&tokens[1])?;

    let mut blur = 0.0;
    let mut spread = 0.0;
    let mut color = Color::BLACK;
    let mut idx = 2;

    if idx < tokens.len() {
        if let Some(v) = parse_length_px(&tokens[idx]) {
            blur = v;
            idx += 1;
        }
    }
    if idx < tokens.len() {
        if let Some(v) = parse_length_px(&tokens[idx]) {
            spread = v;
            idx += 1;
        }
    }
    if idx < tokens.len() {
        if let Some(c) = parse_color(&tokens[idx..].join(" ")) {
            color = c;
        }
    }

    Some(CssProp::BoxShadow(
        Shadow::new().x(x).y(y).blur(blur).spread(spread).color(color),
    ))
}

/// Splits shadow value tokens while keeping `rgba(...)` groups intact.
fn split_shadow_tokens(value: &str) -> Vec<String> {
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
            ' ' if depth == 0 => {
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
    let v = value.trim();
    if v.ends_with('%') {
        v.trim_end_matches('%').parse::<f32>().ok().map(|n| n / 100.0)
    } else {
        v.parse::<f32>().ok()
    }
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
