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
    let fill = parse_fill("conic-gradient(250deg, #ff6432 15%, #ff0000 50%, #ffc0cb 80%)").unwrap();
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
    let fill = parse_fill("linear-gradient(90deg, rgba(0, 0, 0, 0.5) 0%, rgb(255, 255, 255) 100%)")
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

// ── text-decoration / text-overflow / font-style ────────────────────────

#[test]
fn parses_text_decoration_keywords() {
    assert_eq!(parse_text_decoration("none"), Some(TextDecoration::None));
    assert_eq!(
        parse_text_decoration("underline"),
        Some(TextDecoration::Underline)
    );
    assert_eq!(
        parse_text_decoration("overline"),
        Some(TextDecoration::Overline)
    );
    assert_eq!(
        parse_text_decoration("line-through"),
        Some(TextDecoration::LineThrough)
    );
    assert_eq!(parse_text_decoration("nonsense"), None);
}

#[test]
fn parses_text_overflow_keywords_and_custom_string() {
    assert_eq!(parse_text_overflow("clip"), Some(TextOverflow::Clip));
    assert_eq!(
        parse_text_overflow("ellipsis"),
        Some(TextOverflow::Ellipsis)
    );
    assert_eq!(
        parse_text_overflow("\"---\""),
        Some(TextOverflow::Custom("---".to_string()))
    );
    assert_eq!(parse_text_overflow("nonsense"), None);
}

#[test]
fn parses_font_style_keywords() {
    assert_eq!(parse_font_slant("normal"), Some(FontSlant::Upright));
    assert_eq!(parse_font_slant("italic"), Some(FontSlant::Italic));
    assert_eq!(parse_font_slant("oblique"), Some(FontSlant::Oblique));
    assert_eq!(parse_font_slant("nonsense"), None);
}

// ── scale / transform-origin ─────────────────────────────────────────────

#[test]
fn parses_uniform_and_two_axis_scale() {
    assert_eq!(parse_scale("1.5"), Some(Scale::from(1.5)));
    assert_eq!(parse_scale("1.2 0.8"), Some(Scale::from((1.2, 0.8))));
    assert_eq!(parse_scale("banana"), None);
}

#[test]
fn parses_transform_origin_single_keyword() {
    assert_eq!(
        parse_transform_origin("center"),
        Some(TransformOrigin::center())
    );
    assert_eq!(parse_transform_origin("top"), Some(TransformOrigin::top()));
    assert_eq!(
        parse_transform_origin("left"),
        Some(TransformOrigin::left())
    );
}

#[test]
fn parses_transform_origin_compound_keyword_in_either_order() {
    assert_eq!(
        parse_transform_origin("top left"),
        Some(TransformOrigin::top_left())
    );
    assert_eq!(
        parse_transform_origin("left top"),
        Some(TransformOrigin::top_left())
    );
    assert_eq!(
        parse_transform_origin("bottom right"),
        Some(TransformOrigin::bottom_right())
    );
}

#[test]
fn parses_transform_origin_lengths() {
    assert_eq!(
        parse_transform_origin("20% 30%"),
        Some(TransformOrigin {
            x: OriginValue::Fraction(0.2),
            y: OriginValue::Fraction(0.3),
        })
    );
    assert_eq!(
        parse_transform_origin("10px"),
        Some(TransformOrigin {
            x: OriginValue::Pixels(10.0),
            y: OriginValue::Fraction(0.5),
        })
    );
}

// ── position / top / right / bottom / left ──────────────────────────────

#[test]
fn position_absolute_combines_sides_from_separate_declarations() {
    let props = parse_inline("position: absolute; top: 10px; left: 20px;");
    assert_eq!(props.len(), 1);
    match &props[0] {
        CssProp::Position(p) => {
            assert!(p.is_absolute());
            assert_eq!(
                p.pretty(),
                Position::new_absolute().top(10.0).left(20.0).pretty()
            );
        }
        other => panic!("expected Position, got {other:?}"),
    }
}

#[test]
fn position_fixed_maps_to_global() {
    let props = parse_inline("position: fixed; bottom: 5px;");
    assert_eq!(props.len(), 1);
    match &props[0] {
        CssProp::Position(p) => assert!(p.is_global()),
        other => panic!("expected Position, got {other:?}"),
    }
}

#[test]
fn sides_without_a_position_mode_are_dropped() {
    let props = parse_inline("top: 10px; left: 20px; width: 5px;");
    assert_eq!(props, [CssProp::Width(Size::px(5.0))]);
}

#[test]
fn position_static_is_a_no_op() {
    assert!(parse_inline("position: static; top: 10px;").is_empty());
}

#[test]
fn position_relative_is_unsupported() {
    assert!(parse_inline("position: relative; top: 10px;").is_empty());
}
