use freya_core::{
    elements::image::Image,
    prelude::{
        ContainerExt, ContainerPositionExt, ContainerSizeExt, EffectExt, Label, Paragraph, Rect,
        StyleExt, TextStyleExt,
    },
};

use crate::{
    parser::{CssProp, parse_inline},
    stylesheet::StyleSheet,
};

/// Adds `.css()` and `.class()` methods to Freya elements.
///
/// Implemented for each element type with the appropriate subset of CSS properties:
/// - [`Rect`]: full CSS (background, border, shadow, layout, text, effects)
/// - [`Label`], [`Paragraph`]: text + layout CSS (color, font, padding, size)
/// - [`Image`]: layout + effect CSS (size, opacity, rotation, blur)
pub trait CssExt: Sized {
    /// Apply inline CSS declarations to this element.
    ///
    /// ```
    /// rect().css("background: #ff0000; padding: 10px; border-radius: 8px;")
    /// label().css("color: white; font-size: 16px;")
    /// ```
    fn css(self, css: &str) -> Self;

    /// Apply one or more space-separated stylesheet classes to this element.
    ///
    /// Classes must be registered first with [`StyleSheet::register`] or [`StyleSheet::load`].
    fn class(self, classes: &str) -> Self;
}

// ── Rect: full CSS ──────────────────────────────────────────────────────────

impl CssExt for Rect {
    fn css(self, css: &str) -> Self {
        apply_full(self, &parse_inline(css))
    }

    fn class(self, classes: &str) -> Self {
        classes
            .split_whitespace()
            .fold(self, |el, c| apply_full(el, &StyleSheet::get(c)))
    }
}

fn apply_full<T>(element: T, props: &[CssProp]) -> T
where
    T: StyleExt + TextStyleExt + ContainerExt + ContainerSizeExt + EffectExt + Sized,
{
    props.iter().fold(element, |el, prop| match prop {
        CssProp::Width(s) => el.width(s.clone()),
        CssProp::Height(s) => el.height(s.clone()),
        CssProp::MinWidth(s) => el.min_width(s.clone()),
        CssProp::MinHeight(s) => el.min_height(s.clone()),
        CssProp::MaxWidth(s) => el.max_width(s.clone()),
        CssProp::MaxHeight(s) => el.max_height(s.clone()),
        CssProp::Padding(g) => el.padding(*g),
        CssProp::Margin(g) => el.margin(*g),
        CssProp::Background(c) => el.background(*c),
        CssProp::CornerRadius(cr) => el.corner_radius(*cr),
        CssProp::Border(b) => el.border(b.clone()),
        CssProp::BoxShadow(s) => el.shadow(s.clone()),
        CssProp::TextColor(c) => el.color(*c),
        CssProp::FontSize(f) => el.font_size(*f),
        CssProp::FontWeight(w) => el.font_weight(*w),
        CssProp::FontFamily(f) => el.font_family(f.clone()),
        CssProp::TextAlign(ta) => el.text_align(*ta),
        CssProp::Opacity(o) => el.opacity(*o),
        CssProp::Rotation(r) => el.rotation(*r),
        CssProp::Blur(b) => el.blur(*b),
    })
}

// ── Label / Paragraph: text + layout CSS ────────────────────────────────────

impl CssExt for Label {
    fn css(self, css: &str) -> Self {
        apply_text_layout(self, &parse_inline(css))
    }

    fn class(self, classes: &str) -> Self {
        classes
            .split_whitespace()
            .fold(self, |el, c| apply_text_layout(el, &StyleSheet::get(c)))
    }
}

impl CssExt for Paragraph {
    fn css(self, css: &str) -> Self {
        apply_text_layout(self, &parse_inline(css))
    }

    fn class(self, classes: &str) -> Self {
        classes
            .split_whitespace()
            .fold(self, |el, c| apply_text_layout(el, &StyleSheet::get(c)))
    }
}

fn apply_text_layout<T>(element: T, props: &[CssProp]) -> T
where
    T: TextStyleExt + ContainerExt + ContainerSizeExt + Sized,
{
    props.iter().fold(element, |el, prop| match prop {
        CssProp::Width(s) => el.width(s.clone()),
        CssProp::Height(s) => el.height(s.clone()),
        CssProp::MinWidth(s) => el.min_width(s.clone()),
        CssProp::MinHeight(s) => el.min_height(s.clone()),
        CssProp::MaxWidth(s) => el.max_width(s.clone()),
        CssProp::MaxHeight(s) => el.max_height(s.clone()),
        CssProp::Padding(g) => el.padding(*g),
        CssProp::Margin(g) => el.margin(*g),
        CssProp::TextColor(c) => el.color(*c),
        CssProp::FontSize(f) => el.font_size(*f),
        CssProp::FontWeight(w) => el.font_weight(*w),
        CssProp::FontFamily(f) => el.font_family(f.clone()),
        CssProp::TextAlign(ta) => el.text_align(*ta),
        // background, border, shadow, opacity, rotation, blur not supported on text elements
        _ => el,
    })
}

// ── Image: layout + effect CSS ───────────────────────────────────────────────

impl CssExt for Image {
    fn css(self, css: &str) -> Self {
        apply_effect_layout(self, &parse_inline(css))
    }

    fn class(self, classes: &str) -> Self {
        classes
            .split_whitespace()
            .fold(self, |el, c| apply_effect_layout(el, &StyleSheet::get(c)))
    }
}

fn apply_effect_layout<T>(element: T, props: &[CssProp]) -> T
where
    T: EffectExt + ContainerExt + ContainerSizeExt + Sized,
{
    props.iter().fold(element, |el, prop| match prop {
        CssProp::Width(s) => el.width(s.clone()),
        CssProp::Height(s) => el.height(s.clone()),
        CssProp::MinWidth(s) => el.min_width(s.clone()),
        CssProp::MinHeight(s) => el.min_height(s.clone()),
        CssProp::MaxWidth(s) => el.max_width(s.clone()),
        CssProp::MaxHeight(s) => el.max_height(s.clone()),
        CssProp::Padding(g) => el.padding(*g),
        CssProp::Margin(g) => el.margin(*g),
        CssProp::Opacity(o) => el.opacity(*o),
        CssProp::Rotation(r) => el.rotation(*r),
        CssProp::Blur(b) => el.blur(*b),
        _ => el,
    })
}
