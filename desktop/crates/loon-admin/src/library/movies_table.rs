//! Movies table with poster thumbnails.

use egui::{Align, Image, ScrollArea, Ui};
use egui_extras::{Column, TableBuilder};

use crate::api::MovieSummary;

const POSTER_WIDTH: f32 = 40.0;
const ROW_HEIGHT: f32 = 56.0;

/// Sortable movie table widget.
#[derive(Default)]
pub struct MoviesTable {
    sort_title_asc: bool,
}

impl MoviesTable {
    /// Draws the table. Returns the slug when a row is clicked.
    pub fn show(&mut self, ui: &mut Ui, movies: &[MovieSummary]) -> Option<String> {
        let mut movies: Vec<_> = movies.to_vec();
        movies.sort_by(|left, right| {
            let order = left.title.cmp(&right.title);
            if self.sort_title_asc {
                order
            } else {
                order.reverse()
            }
        });

        let mut selected = None;

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(Align::Center))
                    .column(Column::auto().at_least(POSTER_WIDTH + 8.0))
                    .column(Column::remainder().at_least(160.0))
                    .column(Column::auto().at_least(56.0))
                    .column(Column::auto().at_least(72.0))
                    .column(Column::remainder().at_least(200.0))
                    .min_scrolled_height(0.0)
                    .header(24.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Poster");
                        });
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
                            ui.strong("Runtime");
                        });
                        header.col(|ui| {
                            ui.strong("Summary");
                        });
                    })
                    .body(|body| {
                        body.rows(ROW_HEIGHT, movies.len(), |mut row| {
                            let index = row.index();
                            let movie = &movies[index];

                            row.col(|ui| {
                                self.poster_cell(ui, movie);
                            });
                            row.col(|ui| {
                                ui.label(&movie.title);
                            });
                            row.col(|ui| {
                                ui.label(movie.year.map(|y| y.to_string()).unwrap_or_default());
                            });
                            row.col(|ui| {
                                ui.label(format!("{} min", movie.runtime_minutes));
                            });
                            row.col(|ui| {
                                let summary = truncate_summary(&movie.summary, 120);
                                ui.label(summary);
                            });

                            if row.response().clicked() {
                                selected = Some(movie.slug.clone());
                            }
                        });
                    });
            });

        selected
    }

    fn poster_cell(&self, ui: &mut Ui, movie: &MovieSummary) {
        let size = egui::vec2(POSTER_WIDTH, ROW_HEIGHT - 8.0);
        if let Some(url) = movie.poster_url.as_deref().filter(|url| !url.is_empty()) {
            ui.add(
                Image::new(url)
                    .fit_to_exact_size(size)
                    .corner_radius(4.0)
                    .show_loading_spinner(true),
            );
        } else {
            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
            ui.painter().rect_filled(
                rect,
                4.0,
                ui.visuals().faint_bg_color,
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                movie.title.chars().next().unwrap_or('?'),
                egui::FontId::proportional(18.0),
                ui.visuals().weak_text_color(),
            );
        }
    }
}

fn truncate_summary(summary: &str, max_chars: usize) -> String {
    if summary.chars().count() <= max_chars {
        return summary.to_string();
    }
    let truncated: String = summary.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{truncated}…")
}
