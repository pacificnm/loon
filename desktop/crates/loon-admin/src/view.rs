//! Root admin shell view.

use egui::{CentralPanel, SidePanel, TopBottomPanel};
use loon_egui_theme::{EguiThemeAdapter, ThemeAdapter};
use nest_core::AppContext;
use nest_error::NestResult;
use nest_gui::GuiView;
use nest_theme::ThemeService;

use crate::config::LoonAdminConfig;
use crate::library::LibraryPanel;

/// Primary navigation sections for the admin shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum AdminSection {
    /// Library overview and movie list.
    #[default]
    Library,
    /// Scan controls and progress.
    Scan,
    /// Server connection and paths.
    Settings,
}

/// Admin shell with library table and detail navigation.
pub struct AdminView {
    section: AdminSection,
    theme_applied: bool,
    server_url: Option<String>,
    library: LibraryPanel,
}

impl Default for AdminView {
    fn default() -> Self {
        Self {
            section: AdminSection::Library,
            theme_applied: false,
            server_url: None,
            library: LibraryPanel::default(),
        }
    }
}

impl GuiView for AdminView {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &AppContext) -> NestResult<()> {
        self.apply_theme(ui, ctx)?;
        self.ensure_server_url(ctx)?;

        TopBottomPanel::top("admin-top").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Loon Admin");
                ui.separator();
                ui.label("Desktop library manager");
                if let Some(url) = self.server_url.as_deref() {
                    ui.separator();
                    ui.label(url);
                }
            });
        });

        SidePanel::left("admin-nav")
            .default_width(220.0)
            .show_inside(ui, |ui| {
                ui.heading("Sections");
                ui.separator();
                self.nav_button(ui, AdminSection::Library, "Library");
                self.nav_button(ui, AdminSection::Scan, "Scan");
                self.nav_button(ui, AdminSection::Settings, "Settings");
            });

        CentralPanel::default().show_inside(ui, |ui| {
            match self.section {
                AdminSection::Library => {
                    if let Some(server_url) = self.server_url.clone() {
                        if let Err(error) = self.library.ui(ui, ctx, &server_url) {
                            ui.colored_label(ui.visuals().error_fg_color, error.to_string());
                        }
                    } else {
                        ui.label("Server URL not configured.");
                    }
                }
                AdminSection::Scan => self.scan_panel(ui),
                AdminSection::Settings => self.settings_panel(ui),
            }
        });

        Ok(())
    }
}

impl AdminView {
    fn apply_theme(&mut self, ui: &mut egui::Ui, ctx: &AppContext) -> NestResult<()> {
        if self.theme_applied {
            return Ok(());
        }

        let themes = ctx.service::<ThemeService>()?;
        let theme = themes.active_theme()?;
        if let Ok(visuals) = EguiThemeAdapter::adapt(&theme) {
            ui.ctx().style_mut(|style| {
                style.visuals = visuals;
            });
            self.theme_applied = true;
        }

        Ok(())
    }

    fn ensure_server_url(&mut self, ctx: &AppContext) -> NestResult<()> {
        if self.server_url.is_none() {
            self.server_url = Some(LoonAdminConfig::from_context(ctx)?.server_url);
        }
        Ok(())
    }

    fn nav_button(&mut self, ui: &mut egui::Ui, section: AdminSection, label: &str) {
        let selected = self.section == section;
        if ui.selectable_label(selected, label).clicked() {
            self.section = section;
        }
    }

    fn scan_panel(&self, ui: &mut egui::Ui) {
        ui.heading("Scan");
        ui.label("Trigger library scans and show progress.");
        ui.add_space(12.0);
        ui.label("Planned: POST /api/library/scan, status from /api/library/status.");
    }

    fn settings_panel(&self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.label("Server URL is read from [loon-admin] in config.toml.");
        ui.add_space(12.0);
        if let Some(url) = self.server_url.as_deref() {
            ui.monospace(url);
        }
    }
}
