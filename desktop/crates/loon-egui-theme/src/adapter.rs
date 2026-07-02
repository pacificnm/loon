//! Maps [`ThemeDefinition`] tokens to [`egui::Visuals`].

use egui::{Color32, CornerRadius, Stroke, Visuals};
use nest_design::{ColorToken, ThemeDefinition, ThemeMode};
use nest_error::NestResult;
use nest_theme::ThemeAdapter;

/// Adapts Nest design tokens into egui visuals for Loon admin.
pub struct EguiThemeAdapter;

impl ThemeAdapter<Visuals> for EguiThemeAdapter {
    fn adapt(theme: &ThemeDefinition) -> NestResult<Visuals> {
        Ok(adapt_visuals(theme))
    }
}

/// Converts a [`ThemeDefinition`] into egui visuals.
pub fn adapt_visuals(theme: &ThemeDefinition) -> Visuals {
    let mut visuals = match theme.mode {
        ThemeMode::Dark => Visuals::dark(),
        ThemeMode::Light => Visuals::light(),
    };

    let colors = &theme.colors;
    let background = color32(&colors.background);
    let foreground = color32(&colors.foreground);
    let primary = color32(&colors.primary);
    let surface = color32(&colors.surface);
    let border = color32(&colors.border);

    visuals.dark_mode = theme.mode == ThemeMode::Dark;
    visuals.panel_fill = surface;
    visuals.window_fill = background;
    visuals.extreme_bg_color = background;
    visuals.faint_bg_color = surface;
    visuals.override_text_color = Some(foreground);
    visuals.widgets.noninteractive.bg_fill = surface;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, foreground);
    visuals.widgets.inactive.bg_fill = surface;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, foreground);
    visuals.widgets.hovered.bg_fill = primary.gamma_multiply(0.25);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, foreground);
    visuals.widgets.active.bg_fill = primary.gamma_multiply(0.45);
    visuals.widgets.active.fg_stroke = Stroke::new(1.5, foreground);
    visuals.widgets.open.bg_fill = surface;
    visuals.selection.bg_fill = primary.gamma_multiply(0.35);
    visuals.selection.stroke = Stroke::new(1.0, primary);
    visuals.hyperlink_color = primary;
    visuals.warn_fg_color = color32(&theme.status.warning);
    visuals.error_fg_color = color32(&theme.status.error);
    visuals.window_corner_radius = corner_radius(theme.radius.md);
    visuals.menu_corner_radius = corner_radius(theme.radius.sm);
    visuals.window_stroke = Stroke::new(1.0, border);

    visuals
}

fn color32(token: &ColorToken) -> Color32 {
    let hex = token.as_str().trim_start_matches('#');
    let parse = |start: usize, end: usize| u8::from_str_radix(&hex[start..end], 16).unwrap_or(0);

    if hex.len() == 8 {
        Color32::from_rgba_unmultiplied(
            parse(0, 2),
            parse(2, 4),
            parse(4, 6),
            parse(6, 8),
        )
    } else {
        Color32::from_rgb(parse(0, 2), parse(2, 4), parse(4, 6))
    }
}

fn corner_radius(value: f32) -> CornerRadius {
    CornerRadius::same(value.round().clamp(0.0, 24.0) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::themes::loon_dark;

    #[test]
    fn adapts_loon_dark_theme() {
        let theme = loon_dark();
        let visuals = EguiThemeAdapter::adapt(&theme).unwrap();
        assert!(visuals.dark_mode);
        assert_eq!(visuals.hyperlink_color, color32(&theme.colors.primary));
    }
}
