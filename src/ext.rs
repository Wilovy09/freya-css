use freya_core::{
    elements::image::Image,
    prelude::{
        AccessibilityExt, AccessibilityId, AccessibilityIdExt, ContainerExt, ContainerPositionExt,
        ContainerSizeExt, ContainerWithContentExt, Cursor, CursorIcon, EffectExt, EventHandlersExt,
        Label, Paragraph, Platform, Rect, State, StyleExt, TextStyleExt, WritableUtils,
    },
};
use torin::prelude::Size;

use crate::{
    parser::{CssProp, parse_inline_cached},
    stylesheet::StyleSheet,
};

/// Resolves a class's props, merging in any `@media`-scoped rules whose condition
/// currently matches. Only reads (and reactively subscribes to) the window size when
/// the class actually has `@media` rules registered, so plain classes never pay for it.
fn resolve_class_props(class: &str) -> Vec<CssProp> {
    let mut props = StyleSheet::get(class);
    if StyleSheet::has_media_rules(class) {
        let width = Platform::get().root_size.read().width;
        props.extend(StyleSheet::get_media(class, width));
    }
    props
}

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
    /// # use freya::prelude::*;
    /// # use freya_css::CssExt;
    /// rect().css("background: #ff0000; padding: 10px; border-radius: 8px;");
    /// label().css("color: white; font-size: 16px;");
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
        apply_full(self, &parse_inline_cached(css))
    }

    fn class(self, classes: &str) -> Self {
        classes
            .split_whitespace()
            .fold(self, |el, c| apply_full(el, &resolve_class_props(c)))
    }
}

fn apply_full<T>(element: T, props: &[CssProp]) -> T
where
    T: StyleExt
        + TextStyleExt
        + ContainerExt
        + ContainerSizeExt
        + ContainerWithContentExt
        + EffectExt
        + EventHandlersExt
        + Sized,
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
        CssProp::Background(c) => el.background(c.clone()),
        CssProp::CornerRadius(cr) => el.corner_radius(*cr),
        CssProp::Border(b) => el.border(b.clone()),
        CssProp::BoxShadow(s) => el.shadow(s.clone()),
        CssProp::TextColor(c) => el.color(c.clone()),
        CssProp::FontSize(f) => el.font_size(*f),
        CssProp::FontWeight(w) => el.font_weight(*w),
        CssProp::FontFamily(f) => el.font_family(f.clone()),
        CssProp::TextAlign(ta) => el.text_align(*ta),
        CssProp::Opacity(o) => el.opacity(*o),
        CssProp::Rotation(r) => el.rotation(*r),
        CssProp::Blur(b) => el.blur(*b),
        CssProp::Gap(g) => el.spacing(*g),
        CssProp::Direction(d) => el.direction(*d),
        CssProp::MainAlign(a) => el.main_align(a.clone()),
        CssProp::CrossAlign(a) => el.cross_align(a.clone()),
        CssProp::Hidden => el.width(Size::px(0.0)).height(Size::px(0.0)),
        CssProp::Overflow(o) => el.overflow(*o),
        CssProp::Cursor(icon) => apply_cursor(el, *icon),
    })
}

/// Sets the platform cursor icon while the pointer is over the element, resetting
/// it back to the default on leave. Note Freya stores a single handler per event
/// name, so combining `cursor` with another `on_pointer_enter`/`on_pointer_leave`
/// binding on the same element (e.g. [`HoverExt::hover_class`]) means whichever is
/// applied last wins.
fn apply_cursor<T: EventHandlersExt>(element: T, icon: CursorIcon) -> T {
    element
        .on_pointer_enter(move |_| Cursor::set(icon))
        .on_pointer_leave(move |_| Cursor::set(CursorIcon::Default))
}

// ── Label / Paragraph: text + layout CSS ────────────────────────────────────

impl CssExt for Label {
    fn css(self, css: &str) -> Self {
        apply_text_layout(self, &parse_inline_cached(css))
    }

    fn class(self, classes: &str) -> Self {
        classes
            .split_whitespace()
            .fold(self, |el, c| apply_text_layout(el, &resolve_class_props(c)))
    }
}

impl CssExt for Paragraph {
    fn css(self, css: &str) -> Self {
        apply_text_layout(self, &parse_inline_cached(css))
    }

    fn class(self, classes: &str) -> Self {
        classes
            .split_whitespace()
            .fold(self, |el, c| apply_text_layout(el, &resolve_class_props(c)))
    }
}

fn apply_text_layout<T>(element: T, props: &[CssProp]) -> T
where
    T: TextStyleExt + ContainerExt + ContainerSizeExt + EventHandlersExt + Sized,
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
        CssProp::TextColor(c) => el.color(c.clone()),
        CssProp::FontSize(f) => el.font_size(*f),
        CssProp::FontWeight(w) => el.font_weight(*w),
        CssProp::FontFamily(f) => el.font_family(f.clone()),
        CssProp::TextAlign(ta) => el.text_align(*ta),
        CssProp::Hidden => el.width(Size::px(0.0)).height(Size::px(0.0)),
        CssProp::Cursor(icon) => apply_cursor(el, *icon),
        // background, border, shadow, opacity, rotation, blur, gap, direction,
        // align-items, justify-content and overflow are not supported on text elements
        _ => el,
    })
}

// ── Image: layout + effect CSS ───────────────────────────────────────────────

impl CssExt for Image {
    fn css(self, css: &str) -> Self {
        apply_effect_layout(self, &parse_inline_cached(css))
    }

    fn class(self, classes: &str) -> Self {
        classes.split_whitespace().fold(self, |el, c| {
            apply_effect_layout(el, &resolve_class_props(c))
        })
    }
}

fn apply_effect_layout<T>(element: T, props: &[CssProp]) -> T
where
    T: EffectExt
        + ContainerExt
        + ContainerSizeExt
        + ContainerWithContentExt
        + EventHandlersExt
        + Sized,
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
        CssProp::Gap(g) => el.spacing(*g),
        CssProp::Direction(d) => el.direction(*d),
        CssProp::MainAlign(a) => el.main_align(a.clone()),
        CssProp::CrossAlign(a) => el.cross_align(a.clone()),
        CssProp::Hidden => el.width(Size::px(0.0)).height(Size::px(0.0)),
        CssProp::Overflow(o) => el.overflow(*o),
        CssProp::Cursor(icon) => apply_cursor(el, *icon),
        _ => el,
    })
}

// ── :hover support ──────────────────────────────────────────────────────────

/// Adds [`hover_class`](HoverExt::hover_class) for `:hover`-reactive classes.
///
/// Blanket-implemented for every element type this crate supports (`Rect`, `Label`,
/// `Paragraph`, `Image`), since all of them implement [`CssExt`] and [`EventHandlersExt`].
pub trait HoverExt: CssExt + EventHandlersExt + Sized {
    /// Apply `class`, and additionally apply the `<class>:hover` ruleset while the
    /// pointer is over the element.
    ///
    /// Register the hover ruleset like any other class, with a `:hover` suffix:
    ///
    /// ```
    /// # use freya_css::StyleSheet;
    /// StyleSheet::load(
    ///     ".btn { background: #333333; }
    ///      .btn:hover { background: #555555; }",
    /// );
    /// ```
    ///
    /// The caller owns the hover flag (typically created with `use_state`), so it composes
    /// with Freya's reactivity: reading it here subscribes the enclosing `render()` to
    /// re-run whenever it flips, just like reading any other [`State`].
    ///
    /// ```
    /// # use freya::prelude::*;
    /// # use freya_css::{CssExt, HoverExt};
    /// fn hoverable_button() -> impl IntoElement {
    ///     let hovered = use_state(|| false);
    ///     rect().hover_class("btn", hovered)
    /// }
    /// ```
    ///
    /// Freya stores a single handler per event name, so this overwrites any
    /// `on_pointer_enter`/`on_pointer_leave` bound separately on the same element —
    /// notably the `cursor` CSS property and [`ActiveExt::active_class`]'s
    /// `on_pointer_leave`. Put `cursor` inside the class's own ruleset instead of a
    /// separate `.css("cursor: ...")` call, and avoid combining `hover_class` with
    /// `active_class` on the same element.
    fn hover_class(self, class: &str, mut hovered: State<bool>) -> Self {
        let mut element = self.class(class);
        if *hovered.read() {
            element = element.class(&format!("{class}:hover"));
        }
        element
            .on_pointer_enter(move |_| hovered.set(true))
            .on_pointer_leave(move |_| hovered.set(false))
    }
}

impl<T: CssExt + EventHandlersExt> HoverExt for T {}

// ── :active support ─────────────────────────────────────────────────────────

/// Adds [`active_class`](ActiveExt::active_class) for `:active`-reactive classes.
pub trait ActiveExt: CssExt + EventHandlersExt + Sized {
    /// Apply `class`, and additionally apply the `<class>:active` ruleset while the
    /// element is pressed.
    ///
    /// Register the ruleset with an `:active` suffix, same as [`HoverExt::hover_class`]:
    ///
    /// ```
    /// # use freya_css::StyleSheet;
    /// StyleSheet::load(
    ///     ".btn { background: #333333; }
    ///      .btn:active { background: #111111; }",
    /// );
    /// ```
    ///
    /// ```
    /// # use freya::prelude::*;
    /// # use freya_css::{ActiveExt, CssExt};
    /// fn pressable_button() -> impl IntoElement {
    ///     let active = use_state(|| false);
    ///     rect().active_class("btn", active)
    /// }
    /// ```
    ///
    /// Mouse-only: Freya has no unified pointer "release" event that also covers touch,
    /// so this uses `on_mouse_down`/`on_mouse_up`. `on_pointer_leave` resets the active
    /// state if the button is released after dragging off the element.
    ///
    /// Shares the single-handler-per-event limitation described on
    /// [`HoverExt::hover_class`] — don't combine `active_class` with `hover_class` or
    /// the `cursor` property on the same element.
    fn active_class(self, class: &str, mut active: State<bool>) -> Self {
        let mut element = self.class(class);
        if *active.read() {
            element = element.class(&format!("{class}:active"));
        }
        element
            .on_mouse_down(move |_| active.set(true))
            .on_mouse_up(move |_| active.set(false))
            .on_pointer_leave(move |_| active.set(false))
    }
}

impl<T: CssExt + EventHandlersExt> ActiveExt for T {}

// ── :focus support ──────────────────────────────────────────────────────────

/// Adds [`focus_class`](FocusExt::focus_class) for `:focus`-reactive classes.
pub trait FocusExt: CssExt + AccessibilityExt + Sized {
    /// Apply `class`, mark the element focusable and identified by `a11y_id`, and
    /// additionally apply the `<class>:focus` ruleset while it holds focus.
    ///
    /// Register the ruleset with a `:focus` suffix, same as [`HoverExt::hover_class`]:
    ///
    /// ```
    /// # use freya_css::StyleSheet;
    /// StyleSheet::load(
    ///     ".input { border: 1px #333333; }
    ///      .input:focus { border: 1px #89b4fa; }",
    /// );
    /// ```
    ///
    /// Unlike `hover_class`/`active_class`, this does not grant focus itself — it only
    /// reflects whether the element currently has it. Wire up how focus is requested
    /// (e.g. `on_mouse_down` calling `a11y_id.request_focus()`, or keyboard navigation)
    /// separately, exactly as real CSS `:focus` never grants focus on its own:
    ///
    /// ```
    /// # use freya::prelude::*;
    /// # use freya_css::{CssExt, FocusExt};
    /// fn focusable_input() -> impl IntoElement {
    ///     let a11y_id = use_a11y();
    ///     rect()
    ///         .focus_class("input", a11y_id)
    ///         .on_mouse_down(move |_| a11y_id.request_focus())
    /// }
    /// ```
    ///
    /// Doesn't bind any pointer/mouse event, so it composes freely with `hover_class`,
    /// `active_class` and `cursor` on the same element.
    fn focus_class(self, class: &str, a11y_id: AccessibilityId) -> Self {
        let mut element = self.class(class).a11y_id(a11y_id).a11y_focusable(true);
        if a11y_id.is_focused() {
            element = element.class(&format!("{class}:focus"));
        }
        element
    }
}

impl<T: CssExt + AccessibilityExt> FocusExt for T {}
