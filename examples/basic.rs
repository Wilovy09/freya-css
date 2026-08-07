#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use freya::prelude::*;
use freya_css::{CssExt, HoverExt, StyleSheet};

fn main() {
    StyleSheet::load(include_str!("style.css"));

    launch(LaunchConfig::new().with_window(WindowConfig::new(app)))
}

fn app() -> impl IntoElement {
    rect()
        .expanded()
        .center()
        .css("background: #181825;")
        .spacing(16.)
        .child(card_component())
        .child(buttons_row())
}

fn card_component() -> impl IntoElement {
    rect()
        .class("card")
        .css("width: 320px;")
        .spacing(8.)
        .child(
            rect()
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(8.)
                .child(label().class("title").text("freya-css"))
                .child(
                    rect().class("badge").child(
                        label()
                            .css("color: #1e1e2e; font-size: 12px; font-weight: bold;")
                            .text("v0.1"),
                    ),
                ),
        )
        .child(
            label()
                .class("subtitle")
                .text("CSS styling for Freya elements."),
        )
        .child(
            rect()
                .css("background: #313244; border-radius: 6px; padding: 12px; width: fill;")
                .child(
                    label()
                        .css("color: #a6e3a1; font-size: 13px;")
                        .text(r#"rect().class("card").css("opacity: 0.9;")"#),
                ),
        )
}

fn buttons_row() -> impl IntoElement {
    let cancel_hovered = use_state(|| false);

    rect()
        .horizontal()
        .spacing(12.)
        .child(
            rect().class("btn-primary").child(
                label()
                    .css("color: #1e1e2e; font-weight: bold;")
                    .text("Accept"),
            ),
        )
        .child(
            // `.hover_class()` applies `.btn-danger:hover` while the pointer is
            // over the button; `cursor: pointer` (declared on `.btn-danger`) shows
            // a pointer cursor for the same duration.
            rect().hover_class("btn-danger", cancel_hovered).child(
                label()
                    .css("color: #1e1e2e; font-weight: bold;")
                    .text("Cancel"),
            ),
        )
}
