//! Loon-branded egui theme adapter.
//!
//! Implements [`ThemeAdapter`] for `egui::Visuals`, filling the deferred
//! `nest-egui-theme` role for the Loon admin desktop app.

#![deny(missing_docs)]

mod adapter;
mod themes;

pub use adapter::EguiThemeAdapter;
pub use themes::loon_dark;

pub use nest_design::{ThemeDefinition, ThemeId, ThemeMode};
pub use nest_error::{NestError, NestResult};
pub use nest_theme::ThemeAdapter;
