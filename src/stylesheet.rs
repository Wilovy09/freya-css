use std::{
    collections::HashMap,
    io,
    sync::{LazyLock, RwLock},
};

use crate::parser::{CssProp, MediaQuery, StylesheetItem, parse_inline, parse_stylesheet, set_var};

static STYLESHEET: LazyLock<RwLock<HashMap<String, Vec<CssProp>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

type MediaRuleSet = Vec<(MediaQuery, Vec<CssProp>)>;

static MEDIA_RULES: LazyLock<RwLock<HashMap<String, MediaRuleSet>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Global CSS stylesheet registry.
///
/// Register classes at startup and then apply them to elements with `.class("name")`.
pub struct StyleSheet;

impl StyleSheet {
    /// Register a single class with inline CSS declarations.
    ///
    /// ```
    /// # use freya_css::StyleSheet;
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
    /// # use freya_css::StyleSheet;
    /// StyleSheet::load(".btn { background: #0066ff; border-radius: 6px; }");
    /// ```
    /// Load and register all classes from a CSS string.
    ///
    /// Also handles `@media (min-width/max-width: ...)` blocks, `@layer` blocks
    /// (parsed transparently), nested `&:pseudo { ... }` blocks, and a
    /// `:root { --name: value; }` block for CSS variables. See the crate README for
    /// the exact subset of each that's supported.
    pub fn load(css: &str) {
        for item in parse_stylesheet(css) {
            match item {
                StylesheetItem::Rule(class, props) => {
                    if let Ok(mut sheet) = STYLESHEET.write() {
                        sheet.insert(class, props);
                    }
                }
                StylesheetItem::MediaRule(query, class, props) => {
                    if let Ok(mut media) = MEDIA_RULES.write() {
                        media.entry(class).or_default().push((query, props));
                    }
                }
            }
        }
    }

    /// Load and register all classes from a CSS file at runtime.
    pub fn load_file(path: &str) -> io::Result<()> {
        let css = std::fs::read_to_string(path)?;
        Self::load(&css);
        Ok(())
    }

    /// Register a single CSS custom property (`--name`), usable via `var(--name)` in
    /// any later `.css()`/`.class()`/`StyleSheet::load()` call.
    ///
    /// ```
    /// # use freya_css::StyleSheet;
    /// StyleSheet::set_var("primary", "#89b4fa");
    /// StyleSheet::register("btn", "background: var(--primary);");
    /// ```
    pub fn set_var(name: &str, value: &str) {
        set_var(name, value);
    }

    /// Remove a single registered class, if present.
    ///
    /// ```
    /// # use freya_css::StyleSheet;
    /// StyleSheet::register("btn", "background: #0066ff;");
    /// StyleSheet::unregister("btn");
    /// ```
    pub fn unregister(class: &str) {
        if let Ok(mut sheet) = STYLESHEET.write() {
            sheet.remove(class);
        }
        if let Ok(mut media) = MEDIA_RULES.write() {
            media.remove(class);
        }
    }

    /// Remove every registered class, e.g. before loading a different theme.
    ///
    /// ```
    /// # use freya_css::StyleSheet;
    /// StyleSheet::load(".btn { background: #0066ff; }");
    /// StyleSheet::clear();
    /// StyleSheet::load(".btn { background: #ff0066; }");
    /// ```
    pub fn clear() {
        if let Ok(mut sheet) = STYLESHEET.write() {
            sheet.clear();
        }
        if let Ok(mut media) = MEDIA_RULES.write() {
            media.clear();
        }
    }

    pub(crate) fn get(class: &str) -> Vec<CssProp> {
        STYLESHEET
            .read()
            .ok()
            .and_then(|sheet| sheet.get(class).cloned())
            .unwrap_or_default()
    }

    /// Whether `class` has any `@media`-scoped rules registered, so callers can skip
    /// reading the (reactive) window size entirely when it doesn't.
    pub(crate) fn has_media_rules(class: &str) -> bool {
        MEDIA_RULES
            .read()
            .map(|media| media.contains_key(class))
            .unwrap_or(false)
    }

    /// Props from every `@media`-scoped rule on `class` whose condition matches `width`.
    pub(crate) fn get_media(class: &str, width: f32) -> Vec<CssProp> {
        MEDIA_RULES
            .read()
            .ok()
            .and_then(|media| media.get(class).cloned())
            .into_iter()
            .flatten()
            .filter(|(query, _)| query.matches(width))
            .flat_map(|(_, props)| props)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A single test to avoid races on the shared global `STYLESHEET` static
    // when tests run concurrently within the same process.
    #[test]
    fn register_unregister_and_clear_round_trip() {
        StyleSheet::register("stylesheet-test-a", "opacity: 0.5;");
        assert_eq!(StyleSheet::get("stylesheet-test-a").len(), 1);

        StyleSheet::unregister("stylesheet-test-a");
        assert!(StyleSheet::get("stylesheet-test-a").is_empty());

        StyleSheet::register("stylesheet-test-b", "opacity: 0.5;");
        StyleSheet::register("stylesheet-test-c", "opacity: 0.5;");
        assert_eq!(StyleSheet::get("stylesheet-test-b").len(), 1);
        assert_eq!(StyleSheet::get("stylesheet-test-c").len(), 1);

        StyleSheet::clear();
        assert!(StyleSheet::get("stylesheet-test-b").is_empty());
        assert!(StyleSheet::get("stylesheet-test-c").is_empty());
    }
}
