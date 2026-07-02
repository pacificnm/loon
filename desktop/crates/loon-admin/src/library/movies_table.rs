//! Movies admin table.

use egui::{Align, Button, Response, ScrollArea, Ui};
use egui_extras::{Column, TableBuilder};

use crate::api::MovieSummary;

const ROW_HEIGHT: f32 = 30.0;

/// Row action triggered from the actions column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovieRowAction {
    /// Play the movie.
    Play,
    /// Open read-only detail view.
    View,
    /// Open metadata editor.
    Edit,
    /// Remove from library.
    Delete,
}

/// A movie row action with target slug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovieRowEvent {
    /// Movie slug.
    pub slug: String,
    /// Action selected.
    pub action: MovieRowAction,
}

/// Admin movie list table.
#[derive(Default)]
pub struct MoviesTable {
    sort_title_asc: bool,
}

impl MoviesTable {
    /// Draws the table. Returns an event when an action button is clicked.
    pub fn show(&mut self, ui: &mut Ui, movies: &[MovieSummary]) -> Option<MovieRowEvent> {
        let mut movies: Vec<_> = movies.to_vec();
        movies.sort_by(|left, right| {
            let order = left
                .title
                .to_ascii_lowercase()
                .cmp(&right.title.to_ascii_lowercase());
            if self.sort_title_asc {
                order
            } else {
                order.reverse()
            }
        });

        let mut event = None;

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(Align::Center))
                    .column(Column::remainder().at_least(180.0))
                    .column(Column::auto().at_least(52.0))
                    .column(Column::remainder().at_least(240.0))
                    .column(Column::auto().at_least(80.0))
                    .column(Column::auto().at_least(132.0))
                    .min_scrolled_height(0.0)
                    .header(24.0, |mut header| {
                        header.col(|ui| {
                            if ui
                                .button(format!(
                                    "Title {}",
                                    if self.sort_title_asc { "↑" } else { "↓" }
                                ))
                                .clicked()
                            {
                                self.sort_title_asc = !self.sort_title_asc;
                            }
                        });
                        header.col(|ui| {
                            ui.strong("Year");
                        });
                        header.col(|ui| {
                            ui.strong("File");
                        });
                        header.col(|ui| {
                            ui.strong("Size");
                        });
                        header.col(|ui| {
                            ui.strong("Actions");
                        });
                    })
                    .body(|body| {
                        body.rows(ROW_HEIGHT, movies.len(), |mut row| {
                            let index = row.index();
                            let movie = &movies[index];

                            row.col(|ui| {
                                ui.label(&movie.title);
                            });
                            row.col(|ui| {
                                ui.label(movie.year.map(|y| y.to_string()).unwrap_or_else(|| "—".into()));
                            });
                            row.col(|ui| {
                                ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                                let response = ui.label(&movie.relative_path);
                                response.on_hover_text(&movie.relative_path);
                                ui.style_mut().override_text_style = None;
                            });
                            row.col(|ui| {
                                ui.label(format_file_size(movie.size_bytes));
                            });
                            row.col(|ui| {
                                if let Some(clicked) = self.actions_cell(ui, &movie.slug) {
                                    event = Some(clicked);
                                }
                            });
                        });
                    });
            });

        event
    }

    fn actions_cell(&self, ui: &mut Ui, slug: &str) -> Option<MovieRowEvent> {
        let mut event = None;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            if action_button(ui, "▶", "Play").clicked() {
                event = Some(MovieRowEvent {
                    slug: slug.to_string(),
                    action: MovieRowAction::Play,
                });
            }
            if action_button(ui, "◎", "View").clicked() {
                event = Some(MovieRowEvent {
                    slug: slug.to_string(),
                    action: MovieRowAction::View,
                });
            }
            if action_button(ui, "✎", "Edit").clicked() {
                event = Some(MovieRowEvent {
                    slug: slug.to_string(),
                    action: MovieRowAction::Edit,
                });
            }
            if action_button(ui, "✕", "Delete").clicked() {
                event = Some(MovieRowEvent {
                    slug: slug.to_string(),
                    action: MovieRowAction::Delete,
                });
            }
        });
        event
    }
}

fn action_button(ui: &mut Ui, icon: &str, tooltip: &str) -> Response {
    ui.add(Button::new(icon).min_size(egui::vec2(28.0, 24.0)))
        .on_hover_text(tooltip)
}

fn format_file_size(size_bytes: Option<u64>) -> String {
    let Some(bytes) = size_bytes else {
        return "—".to_string();
    };

    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let value = bytes as f64;

    if value >= GB {
        format!("{:.2} GB", value / GB)
    } else if value >= MB {
        format!("{:.1} MB", value / MB)
    } else if value >= KB {
        format!("{:.0} KB", value / KB)
    } else {
        format!("{bytes} B")
    }
}
