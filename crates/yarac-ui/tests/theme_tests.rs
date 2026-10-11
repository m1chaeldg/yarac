use yarac_ui::theme::*;

#[test]
fn test_apple_design_theme_tokens() {
    let theme = Theme::apple_default();
    
    // Non-negotiable Action Blue
    assert_eq!(theme.colors.primary, "#0066cc");
    assert_eq!(theme.colors.primary_focus, "#0071e3");
    assert_eq!(theme.colors.primary_on_dark, "#2997ff");
    
    // Core Canvases
    assert_eq!(theme.colors.canvas, "#ffffff");
    assert_eq!(theme.colors.canvas_parchment, "#f5f5f7");
    assert_eq!(theme.colors.surface_tile_1, "#272729");
    assert_eq!(theme.colors.surface_black, "#000000");

    // Inks
    assert_eq!(theme.colors.ink, "#1d1d1f");
    assert_eq!(theme.colors.body, "#1d1d1f");
    assert_eq!(theme.colors.on_dark, "#ffffff");

    // Radii
    assert_eq!(theme.radii.pill, 9999.0);
    assert_eq!(theme.radii.card_lg, 18.0);
    assert_eq!(theme.radii.btn_sm, 8.0);
    assert_eq!(theme.radii.capsule_md, 11.0);
}

#[test]
fn test_http_method_badge_colors() {
    let theme = Theme::apple_default();
    assert_eq!(theme.method_color("GET"), "#0066cc");    // Apple Action Blue
    assert_eq!(theme.method_color("POST"), "#107c41");   // Green
    assert_eq!(theme.method_color("PUT"), "#b15c00");    // Amber
    assert_eq!(theme.method_color("DELETE"), "#d83b01"); // Red
}
