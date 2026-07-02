//! Root admin shell view.

use egui::{CentralPanel, SidePanel, TopBottomPanel};
use loon_egui_theme::{EguiThemeAdapter, ThemeAdapter};
use nest_core::AppContext;
use nest_error::NestResult;
use nest_gui::GuiView;
use nest_theme::ThemeService;

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

/// Placeholder admin shell for planning and layout iteration.
#[derive(Default)]
pub struct AdminView {
    section: AdminSection,
    theme_applied: bool,
}

impl GuiView for AdminView {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &AppContext) -> NestResult<()> {
        self.apply_theme(ui, ctx)?;

        TopBottomPanel::top("admin-top").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Loon Admin");
                ui.separator();
                ui.label("Desktop library manager");
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
                AdminSection::Library => self.library_panel(ui),
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

    fn nav_button(&mut self, ui: &mut egui::Ui, section: AdminSection, label: &str) {
        let selected = self.section == section;
        if ui.selectable_label(selected, label).clicked() {
            self.section = section;
        }
    }

    fn library_panel(&self, ui: &mut egui::Ui) {
        ui.heading("Library");
        ui.label("Movie browse and metadata editing will live here.");
        ui.add_space(12.0);
        ui.label("Planned: connect to loon-server, list movies, open detail panel.");
    }

    fn scan_panel(&self, ui: &mut egui::Ui) {
        ui.heading("Scan");
        ui.label("Trigger library scans and show progress.");
        ui.add_space(12.0);
        ui.label("Planned: POST /api/library/scan, status from /api/library/status.");
    }

    fn settings_panel(&self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.label("Server URL and connection settings.");
        ui.add_space(12.0);
        ui.label("Planned: [loon-admin] server_url config and health check.");
    }
}
