use freya_core::events::name::EventName;
use freya_css::{ActiveExt, CssExt, FocusExt, HoverExt, StyleSheet};
use freya_testing::prelude::*;
use torin::prelude::{Alignment, Direction, Length, Position, Size, Size2D};

#[test]
fn class_applies_solid_background_to_rect() {
    StyleSheet::register("integration-solid-bg", "background: #1e1e2e;");

    let runner = launch_test(|| rect().class("integration-solid-bg"));

    let expected = Fill::from(Color::from_rgb(0x1e, 0x1e, 0x2e));
    let found =
        runner.find(|_node, element| (element.style().background == expected).then_some(()));

    assert!(
        found.is_some(),
        "expected a Rect with the registered background"
    );
}

#[test]
fn inline_css_applies_gradient_background_to_rect() {
    let runner =
        launch_test(|| rect().css("background: linear-gradient(90deg, #ff0000 0%, #0000ff 100%);"));

    let expected: Fill = LinearGradient::new()
        .angle(90.0)
        .stop((Color::from_rgb(255, 0, 0), 0.0))
        .stop((Color::from_rgb(0, 0, 255), 100.0))
        .into();

    let found =
        runner.find(|_node, element| (element.style().background == expected).then_some(()));

    assert!(
        found.is_some(),
        "expected a Rect with the gradient background"
    );
}

#[test]
fn class_applies_gradient_text_color_to_label() {
    StyleSheet::register(
        "integration-gradient-text",
        "color: linear-gradient(90deg, red 0%, blue 100%);",
    );

    let runner = launch_test(|| label().class("integration-gradient-text").text("hi"));

    let expected: Fill = LinearGradient::new()
        .angle(90.0)
        .stop((Color::RED, 0.0))
        .stop((Color::BLUE, 100.0))
        .into();

    let found = runner.find(|_node, element| {
        (element.text_style().color.as_ref() == Some(&expected)).then_some(())
    });

    assert!(
        found.is_some(),
        "expected a Label with the gradient text color"
    );
}

#[test]
fn inline_css_applies_flex_layout_properties_to_rect() {
    let runner = launch_test(|| {
        rect().css(
            "flex-direction: row; gap: 12px; align-items: center; justify-content: space-between;",
        )
    });

    let found = runner.find(|_node, element| {
        let layout = element.layout();
        (layout.direction == Direction::Horizontal
            && layout.spacing == Length::new(12.0)
            && layout.cross_alignment == Alignment::Center
            && layout.main_alignment == Alignment::SpaceBetween)
            .then_some(())
    });

    assert!(
        found.is_some(),
        "expected a Rect with the flex layout properties applied"
    );
}

#[test]
fn display_none_forces_zero_size() {
    let runner = launch_test(|| rect().css("width: 200px; height: 100px; display: none;"));

    let found = runner.find(|_node, element| {
        let layout = element.layout();
        (layout.width == Size::px(0.0) && layout.height == Size::px(0.0)).then_some(())
    });

    assert!(found.is_some(), "expected display:none to force a 0x0 size");
}

#[test]
fn overflow_hidden_maps_to_clip_on_rect() {
    let runner = launch_test(|| rect().css("overflow: hidden;"));

    let found = runner.find(|_node, element| {
        element
            .effect()
            .filter(|effect| effect.overflow == Overflow::Clip)
            .map(|_| ())
    });

    assert!(
        found.is_some(),
        "expected overflow:hidden to map to Overflow::Clip"
    );
}

#[test]
fn cursor_wires_pointer_enter_and_leave_handlers() {
    let runner = launch_test(|| rect().css("cursor: pointer;"));

    let found = runner.find(|_node, element| {
        element
            .events_handlers()
            .filter(|handlers| {
                handlers.contains_key(&EventName::PointerEnter)
                    && handlers.contains_key(&EventName::PointerLeave)
            })
            .map(|_| ())
    });

    assert!(
        found.is_some(),
        "expected cursor to wire on_pointer_enter/on_pointer_leave"
    );
}

#[test]
fn hover_class_applies_base_only_when_not_hovered() {
    StyleSheet::load(
        ".hover-btn-a { background: #333333; }
         .hover-btn-a:hover { background: #555555; }",
    );

    let runner = launch_test(|| {
        let hovered = use_state(|| false);
        rect().hover_class("hover-btn-a", hovered)
    });

    let expected = Fill::from(Color::from_rgb(0x33, 0x33, 0x33));
    let found =
        runner.find(|_node, element| (element.style().background == expected).then_some(()));

    assert!(
        found.is_some(),
        "expected only the base ruleset while not hovered"
    );
}

#[test]
fn hover_class_applies_hover_ruleset_when_hovered() {
    StyleSheet::load(
        ".hover-btn-b { background: #333333; }
         .hover-btn-b:hover { background: #555555; }",
    );

    let runner = launch_test(|| {
        let hovered = use_state(|| true);
        rect().hover_class("hover-btn-b", hovered)
    });

    let expected = Fill::from(Color::from_rgb(0x55, 0x55, 0x55));
    let found =
        runner.find(|_node, element| (element.style().background == expected).then_some(()));

    assert!(
        found.is_some(),
        "expected the :hover ruleset applied on top of the base one"
    );
}

#[test]
fn hover_class_wires_pointer_enter_and_leave_handlers() {
    StyleSheet::register("hover-btn-c", "background: #333333;");

    let runner = launch_test(|| {
        let hovered = use_state(|| false);
        rect().hover_class("hover-btn-c", hovered)
    });

    let found = runner.find(|_node, element| {
        element
            .events_handlers()
            .filter(|handlers| {
                handlers.contains_key(&EventName::PointerEnter)
                    && handlers.contains_key(&EventName::PointerLeave)
            })
            .map(|_| ())
    });

    assert!(
        found.is_some(),
        "expected hover_class to wire on_pointer_enter/on_pointer_leave"
    );
}

#[test]
fn active_class_applies_base_only_when_not_active() {
    StyleSheet::load(
        ".active-btn-a { background: #333333; }
         .active-btn-a:active { background: #111111; }",
    );

    let runner = launch_test(|| {
        let active = use_state(|| false);
        rect().active_class("active-btn-a", active)
    });

    let expected = Fill::from(Color::from_rgb(0x33, 0x33, 0x33));
    let found =
        runner.find(|_node, element| (element.style().background == expected).then_some(()));

    assert!(
        found.is_some(),
        "expected only the base ruleset while not active"
    );
}

#[test]
fn active_class_applies_active_ruleset_when_active() {
    StyleSheet::load(
        ".active-btn-b { background: #333333; }
         .active-btn-b:active { background: #111111; }",
    );

    let runner = launch_test(|| {
        let active = use_state(|| true);
        rect().active_class("active-btn-b", active)
    });

    let expected = Fill::from(Color::from_rgb(0x11, 0x11, 0x11));
    let found =
        runner.find(|_node, element| (element.style().background == expected).then_some(()));

    assert!(
        found.is_some(),
        "expected the :active ruleset applied on top of the base one"
    );
}

#[test]
fn active_class_wires_mouse_down_up_and_pointer_leave_handlers() {
    StyleSheet::register("active-btn-c", "background: #333333;");

    let runner = launch_test(|| {
        let active = use_state(|| false);
        rect().active_class("active-btn-c", active)
    });

    let found = runner.find(|_node, element| {
        element
            .events_handlers()
            .filter(|handlers| {
                handlers.contains_key(&EventName::MouseDown)
                    && handlers.contains_key(&EventName::MouseUp)
                    && handlers.contains_key(&EventName::PointerLeave)
            })
            .map(|_| ())
    });

    assert!(
        found.is_some(),
        "expected active_class to wire on_mouse_down/on_mouse_up/on_pointer_leave"
    );
}

#[test]
fn focus_class_applies_base_only_when_not_focused() {
    StyleSheet::load(
        ".input-a { background: #333333; }
         .input-a:focus { background: #89b4fa; }",
    );

    let runner = launch_test(|| {
        let a11y_id = use_a11y();
        rect().focus_class("input-a", a11y_id)
    });

    let expected = Fill::from(Color::from_rgb(0x33, 0x33, 0x33));
    let found =
        runner.find(|_node, element| (element.style().background == expected).then_some(()));

    assert!(
        found.is_some(),
        "expected only the base ruleset while not focused"
    );
}

#[test]
fn focus_class_applies_focus_ruleset_when_focused() {
    StyleSheet::load(
        ".input-b { background: #333333; }
         .input-b:focus { background: #89b4fa; }",
    );

    let mut runner = launch_test(|| {
        let a11y_id = use_a11y();
        use_hook(move || a11y_id.request_focus());
        rect().focus_class("input-b", a11y_id)
    });
    // First pass resolves the requested focus strategy into `focused_accessibility_id`;
    // second pass re-renders the component now that the state it reads has changed.
    runner.sync_and_update();
    runner.sync_and_update();

    let expected = Fill::from(Color::from_rgb(0x89, 0xb4, 0xfa));
    let found =
        runner.find(|_node, element| (element.style().background == expected).then_some(()));

    assert!(
        found.is_some(),
        "expected the :focus ruleset applied once the element is focused"
    );
}

#[test]
fn set_var_resolves_in_a_class_applied_to_rect() {
    StyleSheet::set_var("integration-primary", "#123456");
    StyleSheet::register("var-btn", "background: var(--integration-primary);");

    let runner = launch_test(|| rect().class("var-btn"));

    let expected = Fill::from(Color::from_rgb(0x12, 0x34, 0x56));
    let found =
        runner.find(|_node, element| (element.style().background == expected).then_some(()));

    assert!(found.is_some(), "expected the var() reference to resolve");
}

#[test]
fn media_query_applies_on_a_wide_window() {
    StyleSheet::load(
        ".responsive-a { background: #ff0000; }
         @media (min-width: 600px) { .responsive-a { background: #0000ff; } }",
    );

    let (runner, _) = TestingRunner::new(
        || rect().class("responsive-a"),
        Size2D::new(800.0, 600.0),
        |_| {},
        1.0,
    );

    let expected = Fill::from(Color::from_rgb(0, 0, 255));
    let found =
        runner.find(|_node, element| (element.style().background == expected).then_some(()));

    assert!(
        found.is_some(),
        "expected the @media rule to apply on an 800px-wide window"
    );
}

#[test]
fn media_query_does_not_apply_on_a_narrow_window() {
    StyleSheet::load(
        ".responsive-b { background: #ff0000; }
         @media (min-width: 600px) { .responsive-b { background: #0000ff; } }",
    );

    let (runner, _) = TestingRunner::new(
        || rect().class("responsive-b"),
        Size2D::new(300.0, 300.0),
        |_| {},
        1.0,
    );

    let expected = Fill::from(Color::from_rgb(255, 0, 0));
    let found =
        runner.find(|_node, element| (element.style().background == expected).then_some(()));

    assert!(
        found.is_some(),
        "expected only the base rule (not @media) on a 300px-wide window"
    );
}

#[test]
fn inline_css_applies_text_decoration_to_label() {
    let runner = launch_test(|| label().css("text-decoration: underline;").text("hi"));

    let found = runner.find(|_node, element| {
        (element.text_style().text_decoration == Some(TextDecoration::Underline)).then_some(())
    });

    assert!(
        found.is_some(),
        "expected text-decoration:underline on the label"
    );
}

#[test]
fn inline_css_applies_font_style_italic_to_label() {
    let runner = launch_test(|| label().css("font-style: italic;").text("hi"));

    let found = runner.find(|_node, element| {
        (element.text_style().font_slant == Some(FontSlant::Italic)).then_some(())
    });

    assert!(found.is_some(), "expected font-style:italic on the label");
}

#[test]
fn inline_css_applies_scale_to_rect() {
    let runner = launch_test(|| rect().css("scale: 1.2 0.8;"));

    let found = runner.find(|_node, element| {
        element
            .effect()
            .filter(|effect| effect.scale == Some(Scale::from((1.2, 0.8))))
            .map(|_| ())
    });

    assert!(
        found.is_some(),
        "expected scale:1.2 0.8 applied to the rect"
    );
}

#[test]
fn inline_css_applies_transform_origin_to_rect() {
    let runner = launch_test(|| rect().css("transform-origin: top left;"));

    let found = runner.find(|_node, element| {
        element
            .effect()
            .filter(|effect| effect.transform_origin == TransformOrigin::top_left())
            .map(|_| ())
    });

    assert!(
        found.is_some(),
        "expected transform-origin:top left applied to the rect"
    );
}

#[test]
fn inline_css_applies_absolute_position_with_offsets_to_rect() {
    let runner = launch_test(|| rect().css("position: absolute; top: 10px; left: 20px;"));

    let expected = Position::new_absolute().top(10.0).left(20.0);
    let found = runner.find(|_node, element| (element.layout().position == expected).then_some(()));

    assert!(
        found.is_some(),
        "expected position:absolute with top/left offsets applied to the rect"
    );
}
