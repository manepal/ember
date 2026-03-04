use crate::context::{Color, GuiContext, WidgetResponse};
use crate::style::WidgetStyle;
use glam::Vec2;

// ---------------------------------------------------------------------------
// Panel
// ---------------------------------------------------------------------------

/// Draw a rectangular panel (container background).
///
/// This is a purely visual widget — it doesn't track interaction.
/// Use it as a backdrop for groups of widgets.
pub fn panel(ctx: &mut GuiContext, pos: Vec2, size: Vec2, style: &WidgetStyle) {
    ctx.frame.draw_rect(
        pos,
        size,
        style.bg_color,
        style.border_color,
        style.border_width,
        style.corner_radius,
    );
}

// ---------------------------------------------------------------------------
// Label
// ---------------------------------------------------------------------------

/// Draw a text label at the given position.
///
/// Labels are non-interactive — they only emit a text draw command.
pub fn label(ctx: &mut GuiContext, pos: Vec2, text: &str, style: &WidgetStyle) {
    ctx.frame
        .draw_text(pos, text.to_string(), style.font_size, style.text_color);
}

// ---------------------------------------------------------------------------
// Button
// ---------------------------------------------------------------------------

/// Draw an interactive button and return its response.
///
/// The button changes color on hover and press, and returns a `WidgetResponse`
/// indicating whether it was clicked, hovered, etc.
pub fn button(
    ctx: &mut GuiContext,
    label_text: &str,
    pos: Vec2,
    size: Vec2,
    style: &WidgetStyle,
) -> WidgetResponse {
    let id = ctx.make_id(label_text);
    let response = ctx.widget_interaction(id, pos, size);

    // Choose background color based on interaction state
    let bg = if response.active {
        style.bg_color_active
    } else if response.hovered {
        style.bg_color_hovered
    } else {
        style.bg_color
    };

    // Draw button background
    ctx.frame.draw_rect(
        pos,
        size,
        bg,
        style.border_color,
        style.border_width,
        style.corner_radius,
    );

    // Draw centered label text
    let text_x = pos.x + style.padding;
    let text_y = pos.y + (size.y - style.font_size) * 0.5;
    ctx.frame.draw_text(
        Vec2::new(text_x, text_y),
        label_text.to_string(),
        style.font_size,
        style.text_color,
    );

    response
}

// ---------------------------------------------------------------------------
// Checkbox
// ---------------------------------------------------------------------------

/// Draw a checkbox and return whether the value changed.
///
/// `value` is the current state of the checkbox (true = checked).
/// Returns `true` in the `clicked` field of the response when the user toggles it.
pub fn checkbox(
    ctx: &mut GuiContext,
    label_text: &str,
    value: bool,
    pos: Vec2,
    style: &WidgetStyle,
) -> WidgetResponse {
    let box_size = Vec2::new(style.font_size + 4.0, style.font_size + 4.0);
    let id = ctx.make_id(label_text);
    let response = ctx.widget_interaction(id, pos, box_size);

    // Draw checkbox box
    let bg = if response.active {
        style.bg_color_active
    } else if response.hovered {
        style.bg_color_hovered
    } else {
        style.bg_color
    };

    ctx.frame.draw_rect(
        pos,
        box_size,
        bg,
        style.border_color,
        style.border_width,
        2.0,
    );

    // Draw checkmark if checked
    if value {
        let inner_padding = 3.0;
        let inner_size = box_size - Vec2::splat(inner_padding * 2.0);
        ctx.frame.draw_rect(
            pos + Vec2::splat(inner_padding),
            inner_size,
            style.text_color,
            Color::TRANSPARENT,
            0.0,
            1.0,
        );
    }

    // Draw label text next to checkbox
    let text_x = pos.x + box_size.x + style.padding;
    let text_y = pos.y + (box_size.y - style.font_size) * 0.5;
    ctx.frame.draw_text(
        Vec2::new(text_x, text_y),
        label_text.to_string(),
        style.font_size,
        style.text_color,
    );

    response
}

// ---------------------------------------------------------------------------
// Slider
// ---------------------------------------------------------------------------

/// A slider result, containing the widget response and the new value.
pub struct SliderResult {
    pub response: WidgetResponse,
    pub value: f32,
}

/// Draw a horizontal slider and return the result with the updated value.
///
/// `value` should be in the range `[min, max]`. The slider displays a track
/// and a draggable thumb.
pub fn slider(
    ctx: &mut GuiContext,
    label_text: &str,
    value: f32,
    min: f32,
    max: f32,
    pos: Vec2,
    width: f32,
    style: &WidgetStyle,
) -> SliderResult {
    let height = style.font_size + style.padding * 2.0;
    let size = Vec2::new(width, height);
    let id = ctx.make_id(label_text);
    let response = ctx.widget_interaction(id, pos, size);

    // Calculate thumb position
    let range = max - min;
    let _normalized = if range > 0.0 {
        ((value - min) / range).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let track_width = width - style.padding * 2.0;

    // Update value if being dragged
    let new_value = if response.active || response.dragged {
        let local_x = (ctx.cursor_pos.x - pos.x - style.padding).clamp(0.0, track_width);
        let new_normalized = local_x / track_width;
        min + new_normalized * range
    } else {
        value
    };

    // Draw track background
    let track_height = 4.0;
    let track_y = pos.y + (height - track_height) * 0.5;
    ctx.frame.draw_rect(
        Vec2::new(pos.x + style.padding, track_y),
        Vec2::new(track_width, track_height),
        style.bg_color,
        style.border_color,
        0.5,
        2.0,
    );

    // Draw filled portion of track
    let filled_width = ((new_value - min) / range).clamp(0.0, 1.0) * track_width;
    ctx.frame.draw_rect(
        Vec2::new(pos.x + style.padding, track_y),
        Vec2::new(filled_width, track_height),
        style.bg_color_hovered,
        Color::TRANSPARENT,
        0.0,
        2.0,
    );

    // Draw thumb
    let thumb_size = Vec2::new(12.0, height - 4.0);
    let final_thumb_x =
        pos.x + style.padding + ((new_value - min) / range).clamp(0.0, 1.0) * track_width
            - thumb_size.x * 0.5;
    let thumb_bg = if response.active {
        style.bg_color_active
    } else if response.hovered {
        style.bg_color_hovered
    } else {
        style.text_color
    };
    ctx.frame.draw_rect(
        Vec2::new(final_thumb_x, pos.y + 2.0),
        thumb_size,
        thumb_bg,
        style.border_color,
        1.0,
        3.0,
    );

    SliderResult {
        response,
        value: new_value,
    }
}

// ---------------------------------------------------------------------------
// Progress Bar
// ---------------------------------------------------------------------------

/// Draw a progress bar showing a value from 0.0 to 1.0.
pub fn progress_bar(
    ctx: &mut GuiContext,
    progress: f32,
    pos: Vec2,
    width: f32,
    style: &WidgetStyle,
) {
    let height = style.font_size;
    let clamped = progress.clamp(0.0, 1.0);

    // Draw background track
    ctx.frame.draw_rect(
        pos,
        Vec2::new(width, height),
        style.bg_color,
        style.border_color,
        style.border_width,
        style.corner_radius,
    );

    // Draw filled portion
    if clamped > 0.0 {
        let fill_width = width * clamped;
        ctx.frame.draw_rect(
            pos,
            Vec2::new(fill_width, height),
            style.bg_color_hovered,
            Color::TRANSPARENT,
            0.0,
            style.corner_radius,
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::DrawCommand;

    fn default_style() -> WidgetStyle {
        WidgetStyle::default()
    }

    #[test]
    fn panel_emits_rect_command() {
        let mut ctx = GuiContext::new();
        ctx.begin_frame(Vec2::ZERO, false, false, false);

        let style = default_style();
        panel(
            &mut ctx,
            Vec2::new(10.0, 10.0),
            Vec2::new(200.0, 100.0),
            &style,
        );

        assert_eq!(ctx.frame.commands.len(), 1);
        assert!(matches!(ctx.frame.commands[0], DrawCommand::Rect { .. }));
    }

    #[test]
    fn label_emits_text_command() {
        let mut ctx = GuiContext::new();
        ctx.begin_frame(Vec2::ZERO, false, false, false);

        let style = default_style();
        label(&mut ctx, Vec2::new(10.0, 10.0), "Hello World", &style);

        assert_eq!(ctx.frame.commands.len(), 1);
        assert!(matches!(ctx.frame.commands[0], DrawCommand::Text { .. }));
    }

    #[test]
    fn button_emits_rect_and_text() {
        let mut ctx = GuiContext::new();
        ctx.begin_frame(Vec2::ZERO, false, false, false);

        let style = default_style();
        let response = button(
            &mut ctx,
            "Click Me",
            Vec2::new(10.0, 10.0),
            Vec2::new(100.0, 30.0),
            &style,
        );

        // Not hovering → not clicked
        assert!(!response.clicked);
        assert!(!response.hovered);

        // Should emit rect (background) + text (label)
        assert_eq!(ctx.frame.commands.len(), 2);
    }

    #[test]
    fn button_hover_changes_color() {
        let mut ctx = GuiContext::new();
        let style = default_style();
        let pos = Vec2::new(10.0, 10.0);
        let size = Vec2::new(100.0, 30.0);

        // Mouse is hovering over the button
        ctx.begin_frame(Vec2::new(50.0, 25.0), false, false, false);
        let response = button(&mut ctx, "Hover Me", pos, size, &style);

        assert!(response.hovered);
        assert!(!response.clicked);

        // The first draw command should use hover color
        if let DrawCommand::Rect { color, .. } = &ctx.frame.commands[0] {
            assert_eq!(color.r, style.bg_color_hovered.r);
        }
    }

    #[test]
    fn button_click_returns_true() {
        let mut ctx = GuiContext::new();
        let style = default_style();
        let pos = Vec2::new(10.0, 10.0);
        let size = Vec2::new(100.0, 30.0);

        // Frame 1: press
        ctx.begin_frame(Vec2::new(50.0, 25.0), true, true, false);
        button(&mut ctx, "Click Me", pos, size, &style);

        // Frame 2: release while hovering → click
        ctx.begin_frame(Vec2::new(50.0, 25.0), false, false, true);
        let response = button(&mut ctx, "Click Me", pos, size, &style);
        assert!(response.clicked);
    }

    #[test]
    fn checkbox_emits_commands() {
        let mut ctx = GuiContext::new();
        ctx.begin_frame(Vec2::ZERO, false, false, false);

        let style = default_style();
        checkbox(
            &mut ctx,
            "Enable Option",
            false,
            Vec2::new(10.0, 10.0),
            &style,
        );

        // Should emit: box rect + label text (no checkmark when unchecked)
        assert_eq!(ctx.frame.commands.len(), 2);
    }

    #[test]
    fn checkbox_checked_emits_checkmark() {
        let mut ctx = GuiContext::new();
        ctx.begin_frame(Vec2::ZERO, false, false, false);

        let style = default_style();
        checkbox(
            &mut ctx,
            "Enable Option",
            true,
            Vec2::new(10.0, 10.0),
            &style,
        );

        // Should emit: box rect + checkmark rect + label text
        assert_eq!(ctx.frame.commands.len(), 3);
    }

    #[test]
    fn slider_returns_clamped_value() {
        let mut ctx = GuiContext::new();
        ctx.begin_frame(Vec2::ZERO, false, false, false);

        let style = default_style();
        let result = slider(
            &mut ctx,
            "Volume",
            0.5,
            0.0,
            1.0,
            Vec2::new(10.0, 10.0),
            200.0,
            &style,
        );

        // Not dragging, should return original value
        assert!((result.value - 0.5).abs() < 0.001);
    }

    #[test]
    fn progress_bar_emits_track_and_fill() {
        let mut ctx = GuiContext::new();
        ctx.begin_frame(Vec2::ZERO, false, false, false);

        let style = default_style();
        progress_bar(&mut ctx, 0.75, Vec2::new(10.0, 10.0), 200.0, &style);

        // Should emit: track rect + fill rect
        assert_eq!(ctx.frame.commands.len(), 2);
    }

    #[test]
    fn progress_bar_zero_only_track() {
        let mut ctx = GuiContext::new();
        ctx.begin_frame(Vec2::ZERO, false, false, false);

        let style = default_style();
        progress_bar(&mut ctx, 0.0, Vec2::new(10.0, 10.0), 200.0, &style);

        // Zero progress → only track, no fill
        assert_eq!(ctx.frame.commands.len(), 1);
    }
}
