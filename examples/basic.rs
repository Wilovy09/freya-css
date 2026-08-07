#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use freya::prelude::*;
use freya_css::{CssExt, StyleSheet};

fn main() {
    // Register classes once at startup
    StyleSheet::load(
        r#"
        .card {
            background: #1e1e2e;
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 4px 16px rgba(0,0,0,0.4);
        }

        .badge {
            background: #cba6f7;
            border-radius: 99px;
            padding: 4px 12px;
        }

        .title {
            color: #cdd6f4;
            font-size: 22px;
            font-weight: bold;
        }

        .subtitle {
            color: #a6adc8;
            font-size: 14px;
        }

        .btn-primary {
            background: #89b4fa;
            border-radius: 8px;
            padding: 8px 20px;
        }

        .btn-danger {
            background: #f38ba8;
            border-radius: 8px;
            padding: 8px 20px;
        }
        "#,
    );

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
            rect().class("btn-danger").child(
                label()
                    .css("color: #1e1e2e; font-weight: bold;")
                    .text("Cancel"),
            ),
        )
}
