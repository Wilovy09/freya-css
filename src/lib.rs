//! CSS-flavored styling for [Freya](https://docs.rs/freya) elements.
//!
//! Write real CSS syntax — hex colors, gradients, `px`/`%`, shorthand `padding`/`margin`,
//! `box-shadow`, flex layout properties — instead of chaining builder calls by hand.
//!
//! ```
//! # use freya::prelude::*;
//! # use freya_css::CssExt;
//! rect().css("background: #1e1e2e; border-radius: 12px; padding: 20px;");
//! ```
//!
//! Entry points:
//!
//! - [`CssExt::css`] — apply inline CSS declarations directly to an element.
//! - [`CssExt::class`] — apply one or more classes registered in [`StyleSheet`].
//! - [`HoverExt::hover_class`] — like `class`, plus the matching `:hover` ruleset
//!   while the pointer is over the element.
//! - [`ActiveExt::active_class`] — like `class`, plus the matching `:active` ruleset
//!   while the element is pressed.
//! - [`FocusExt::focus_class`] — like `class`, plus the matching `:focus` ruleset
//!   while the element holds accessibility focus.
//!
//! See the [repository README](https://github.com/Wilovy09/freya-css) for the full list of
//! supported properties and known limitations.

mod ext;
mod parser;
mod stylesheet;

pub use ext::{ActiveExt, CssExt, FocusExt, HoverExt};
pub use stylesheet::StyleSheet;
