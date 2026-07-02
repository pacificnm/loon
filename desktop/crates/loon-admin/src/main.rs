//! Loon admin desktop entry point.

mod api;
mod config;
mod library;
mod module;
mod view;

use nest_gui::GuiApp;

use crate::module::LoonThemeModule;
use crate::view::AdminView;

fn main() {
    GuiApp::new("loon-admin")
        .module(LoonThemeModule::loon_dark())
        .view(AdminView::default())
        .run();
}
