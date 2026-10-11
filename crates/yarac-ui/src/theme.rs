//! Apple-design-analysis theme tokens defined per frontend/DESIGN.md
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub primary: String,
    pub primary_focus: String,
    pub primary_on_dark: String,
    pub ink: String,
    pub body: String,
    pub body_on_dark: String,
    pub body_muted: String,
    pub ink_muted_80: String,
    pub ink_muted_48: String,
    pub divider_soft: String,
    pub hairline: String,
    pub canvas: String,
    pub canvas_parchment: String,
    pub surface_pearl: String,
    pub surface_tile_1: String,
    pub surface_tile_2: String,
    pub surface_tile_3: String,
    pub surface_black: String,
    pub surface_chip_translucent: String,
    pub on_primary: String,
    pub on_dark: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeRadii {
    pub none: f32,
    pub xs: f32,
    pub btn_sm: f32,
    pub capsule_md: f32,
    pub card_lg: f32,
    pub pill: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub colors: ThemeColors,
    pub radii: ThemeRadii,
}

impl Theme {
    pub fn apple_default() -> Self {
        Self {
            colors: ThemeColors {
                primary: "#0066cc".into(),
                primary_focus: "#0071e3".into(),
                primary_on_dark: "#2997ff".into(),
                ink: "#1d1d1f".into(),
                body: "#1d1d1f".into(),
                body_on_dark: "#ffffff".into(),
                body_muted: "#cccccc".into(),
                ink_muted_80: "#333333".into(),
                ink_muted_48: "#7a7a7a".into(),
                divider_soft: "#f0f0f0".into(),
                hairline: "#e0e0e0".into(),
                canvas: "#ffffff".into(),
                canvas_parchment: "#f5f5f7".into(),
                surface_pearl: "#fafafc".into(),
                surface_tile_1: "#272729".into(),
                surface_tile_2: "#2a2a2c".into(),
                surface_tile_3: "#252527".into(),
                surface_black: "#000000".into(),
                surface_chip_translucent: "#d2d2d7".into(),
                on_primary: "#ffffff".into(),
                on_dark: "#ffffff".into(),
            },
            radii: ThemeRadii {
                none: 0.0,
                xs: 5.0,
                btn_sm: 8.0,
                capsule_md: 11.0,
                card_lg: 18.0,
                pill: 9999.0,
            },
        }
    }

    pub fn method_color(&self, method: &str) -> &'static str {
        match method.to_uppercase().as_str() {
            "GET" => "#0066cc",
            "POST" => "#107c41",
            "PUT" => "#b15c00",
            "DELETE" => "#d83b01",
            "PATCH" => "#8764b8",
            "HEAD" | "OPTIONS" => "#5c2d91",
            _ => "#7a7a7a",
        }
    }
}
