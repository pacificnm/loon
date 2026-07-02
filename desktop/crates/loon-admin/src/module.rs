//! Registers the Loon dark theme with [`ThemeService`].

use loon_egui_theme::loon_dark;
use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_design::ThemeDefinition;
use nest_theme::ThemeService;

/// Module id for [`LoonThemeModule`].
pub const LOON_THEME_MODULE_ID: ModuleId = ModuleId("loon-theme");

/// Registers the Loon admin theme and sets it active.
pub struct LoonThemeModule {
    theme: ThemeDefinition,
}

impl LoonThemeModule {
    /// Creates a module using the built-in Loon dark theme.
    pub fn loon_dark() -> Self {
        Self {
            theme: loon_dark(),
        }
    }
}

impl Module for LoonThemeModule {
    fn id(&self) -> ModuleId {
        LOON_THEME_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let service = ThemeService::new();
        service.register_theme(self.theme.clone())?;
        service.set_active_theme(&self.theme.id)?;
        app.register_service(service)
    }
}
