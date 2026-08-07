use std::{
    collections::HashMap,
    io,
    sync::{LazyLock, RwLock},
};

use crate::parser::{parse_inline, parse_stylesheet, CssProp};

static STYLESHEET: LazyLock<RwLock<HashMap<String, Vec<CssProp>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Global CSS stylesheet registry.
///
/// Register classes at startup and then apply them to elements with `.class("name")`.
pub struct StyleSheet;

impl StyleSheet {
    /// Register a single class with inline CSS declarations.
    ///
    /// ```
    /// StyleSheet::register("btn", "background: #0066ff; border-radius: 6px; padding: 8px 16px;");
    /// ```
    pub fn register(class: &str, css: &str) {
        if let Ok(mut sheet) = STYLESHEET.write() {
            sheet.insert(class.to_string(), parse_inline(css));
        }
    }

    /// Load and register all classes from a CSS string.
    ///
    /// Suitable for use with `include_str!("styles.css")` for zero-overhead embedding.
    ///
    /// ```
    /// StyleSheet::load(include_str!("styles.css"));
    /// ```
    pub fn load(css: &str) {
        if let Ok(mut sheet) = STYLESHEET.write() {
            for (class, props) in parse_stylesheet(css) {
                sheet.insert(class, props);
            }
        }
    }

    /// Load and register all classes from a CSS file at runtime.
    pub fn load_file(path: &str) -> io::Result<()> {
        let css = std::fs::read_to_string(path)?;
        Self::load(&css);
        Ok(())
    }

    pub(crate) fn get(class: &str) -> Vec<CssProp> {
        STYLESHEET
            .read()
            .ok()
            .and_then(|sheet| sheet.get(class).cloned())
            .unwrap_or_default()
    }
}
