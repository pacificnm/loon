//! Loon theme definitions.

use nest_design::{
    theme::{ThemeDefinition, ThemeId, ThemeMode},
    tokens::{
        ColorToken, ColorTokens, RadiusTokens, SpacingTokens, StatusTokens, TypographyStyle,
        TypographyTokens,
    },
};

fn color(value: &str) -> ColorToken {
    ColorToken::new(value).expect("valid theme color")
}

/// Loon admin dark theme aligned with the webOS client palette.
pub fn loon_dark() -> ThemeDefinition {
    ThemeDefinition {
        id: ThemeId::new("loon-dark"),
        mode: ThemeMode::Dark,
        colors: ColorTokens {
            background: color("#090B10"),
            foreground: color("#E2E8F0"),
            primary: color("#7DD3FC"),
            secondary: color("#64748B"),
            border: color("#1E293B"),
            surface: color("#111827"),
            accent: Some(color("#38BDF8")),
            muted: Some(color("#94A3B8")),
        },
        spacing: SpacingTokens {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: Some(48.0),
        },
        radius: RadiusTokens {
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            full: Some(9999.0),
        },
        typography: TypographyTokens {
            body: TypographyStyle {
                font_family: "Inter".to_string(),
                size: 14.0,
                line_height: 20.0,
                weight: 400,
            },
            heading: TypographyStyle {
                font_family: "Inter".to_string(),
                size: 20.0,
                line_height: 28.0,
                weight: 600,
            },
            caption: None,
            mono: None,
        },
        status: StatusTokens {
            success: color("#22C55E"),
            warning: color("#F59E0B"),
            error: color("#EF4444"),
            info: color("#38BDF8"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loon_dark_has_expected_id() {
        let theme = loon_dark();
        assert_eq!(theme.id.as_str(), "loon-dark");
        assert_eq!(theme.mode, ThemeMode::Dark);
    }
}
