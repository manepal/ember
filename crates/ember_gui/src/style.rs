use crate::context::Color;

// ---------------------------------------------------------------------------
// WidgetStyle — per-widget visual overrides
// ---------------------------------------------------------------------------

/// Visual style properties that can be customized per-widget.
#[derive(Debug, Clone)]
pub struct WidgetStyle {
    pub bg_color: Color,
    pub bg_color_hovered: Color,
    pub bg_color_active: Color,
    pub text_color: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub corner_radius: f32,
    pub font_size: f32,
    pub padding: f32,
}

impl Default for WidgetStyle {
    fn default() -> Self {
        Self {
            bg_color: Color::rgba(0.2, 0.2, 0.2, 1.0),
            bg_color_hovered: Color::rgba(0.3, 0.3, 0.35, 1.0),
            bg_color_active: Color::rgba(0.15, 0.15, 0.2, 1.0),
            text_color: Color::WHITE,
            border_color: Color::rgba(0.4, 0.4, 0.45, 1.0),
            border_width: 1.0,
            corner_radius: 4.0,
            font_size: 16.0,
            padding: 8.0,
        }
    }
}

// ---------------------------------------------------------------------------
// GuiTheme — global theming resource
// ---------------------------------------------------------------------------

/// Global theme resource controlling default colors, spacing, and font sizes
/// for all GUI widgets.
pub struct GuiTheme {
    /// Default style for panel backgrounds.
    pub panel: WidgetStyle,
    /// Default style for buttons.
    pub button: WidgetStyle,
    /// Default style for labels/text.
    pub label: WidgetStyle,
    /// Default spacing between widgets in automatic layouts.
    pub spacing: f32,
    /// Default padding inside containers.
    pub padding: f32,
    /// Window/panel background color.
    pub window_bg: Color,
}

impl Default for GuiTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl GuiTheme {
    /// A dark theme with muted blues and grays.
    pub fn dark() -> Self {
        Self {
            panel: WidgetStyle {
                bg_color: Color::rgba(0.12, 0.12, 0.15, 0.95),
                bg_color_hovered: Color::rgba(0.15, 0.15, 0.18, 0.95),
                bg_color_active: Color::rgba(0.10, 0.10, 0.13, 0.95),
                text_color: Color::rgba(0.9, 0.9, 0.92, 1.0),
                border_color: Color::rgba(0.25, 0.25, 0.3, 1.0),
                border_width: 1.0,
                corner_radius: 6.0,
                font_size: 14.0,
                padding: 10.0,
            },
            button: WidgetStyle {
                bg_color: Color::rgba(0.22, 0.22, 0.28, 1.0),
                bg_color_hovered: Color::rgba(0.30, 0.30, 0.40, 1.0),
                bg_color_active: Color::rgba(0.18, 0.18, 0.24, 1.0),
                text_color: Color::WHITE,
                border_color: Color::rgba(0.35, 0.35, 0.45, 1.0),
                border_width: 1.0,
                corner_radius: 4.0,
                font_size: 14.0,
                padding: 8.0,
            },
            label: WidgetStyle {
                bg_color: Color::TRANSPARENT,
                bg_color_hovered: Color::TRANSPARENT,
                bg_color_active: Color::TRANSPARENT,
                text_color: Color::rgba(0.85, 0.85, 0.88, 1.0),
                border_color: Color::TRANSPARENT,
                border_width: 0.0,
                corner_radius: 0.0,
                font_size: 14.0,
                padding: 2.0,
            },
            spacing: 6.0,
            padding: 10.0,
            window_bg: Color::rgba(0.08, 0.08, 0.10, 0.98),
        }
    }

    /// A light theme with subtle grays and white backgrounds.
    pub fn light() -> Self {
        Self {
            panel: WidgetStyle {
                bg_color: Color::rgba(0.95, 0.95, 0.96, 0.98),
                bg_color_hovered: Color::rgba(0.92, 0.92, 0.94, 0.98),
                bg_color_active: Color::rgba(0.88, 0.88, 0.90, 0.98),
                text_color: Color::rgba(0.1, 0.1, 0.12, 1.0),
                border_color: Color::rgba(0.75, 0.75, 0.78, 1.0),
                border_width: 1.0,
                corner_radius: 6.0,
                font_size: 14.0,
                padding: 10.0,
            },
            button: WidgetStyle {
                bg_color: Color::rgba(0.88, 0.88, 0.92, 1.0),
                bg_color_hovered: Color::rgba(0.80, 0.80, 0.88, 1.0),
                bg_color_active: Color::rgba(0.75, 0.75, 0.82, 1.0),
                text_color: Color::rgba(0.1, 0.1, 0.12, 1.0),
                border_color: Color::rgba(0.70, 0.70, 0.74, 1.0),
                border_width: 1.0,
                corner_radius: 4.0,
                font_size: 14.0,
                padding: 8.0,
            },
            label: WidgetStyle {
                bg_color: Color::TRANSPARENT,
                bg_color_hovered: Color::TRANSPARENT,
                bg_color_active: Color::TRANSPARENT,
                text_color: Color::rgba(0.15, 0.15, 0.18, 1.0),
                border_color: Color::TRANSPARENT,
                border_width: 0.0,
                corner_radius: 0.0,
                font_size: 14.0,
                padding: 2.0,
            },
            spacing: 6.0,
            padding: 10.0,
            window_bg: Color::rgba(0.98, 0.98, 0.99, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_theme_creates_valid_defaults() {
        let theme = GuiTheme::dark();
        assert!(theme.spacing > 0.0);
        assert!(theme.button.font_size > 0.0);
        assert!(theme.panel.corner_radius >= 0.0);
    }

    #[test]
    fn light_theme_creates_valid_defaults() {
        let theme = GuiTheme::light();
        assert!(theme.spacing > 0.0);
        assert!(theme.label.text_color.r < 0.5); // dark text on light bg
    }

    #[test]
    fn default_theme_is_dark() {
        let default = GuiTheme::default();
        let dark = GuiTheme::dark();
        // Both should have the same window_bg
        assert_eq!(default.window_bg.r, dark.window_bg.r);
        assert_eq!(default.window_bg.g, dark.window_bg.g);
    }
}
