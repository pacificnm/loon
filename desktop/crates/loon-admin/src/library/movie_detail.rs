//! Movie detail and metadata edit screen (stub).

use egui::{Image, ScrollArea, Ui};

use crate::api::MovieDetail;

const POSTER_WIDTH: f32 = 160.0;

/// Detail panel for a single movie. Editing will be wired in a follow-up.
#[derive(Default)]
pub struct MovieDetailPanel;

impl MovieDetailPanel {
    /// Renders movie metadata and placeholder edit fields.
    pub fn show(&mut self, ui: &mut Ui, movie: &MovieDetail) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_top(|ui| {
                self.poster(ui, movie);
                ui.vertical(|ui| {
                    ui.heading(&movie.title);
                    if let Some(original) = movie.original_title.as_deref() {
                        if original != movie.title {
                            ui.label(format!("Original: {original}"));
                        }
                    }
                    ui.label(format!(
                        "Slug: {}  ·  Year: {}  ·  Runtime: {} min",
                        movie.slug,
                        movie.year.map(|y| y.to_string()).unwrap_or_else(|| "—".into()),
                        movie
                            .runtime_minutes
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "—".into()),
                    ));
                    if !movie.genres.is_empty() {
                        ui.label(format!("Genres: {}", movie.genres.join(", ")));
                    }
                    ui.label(format!(
                        "Favorite: {}",
                        if movie.is_favorite { "yes" } else { "no" }
                    ));
                    ui.label(format!("File: {} ({})", movie.file.filename, movie.file.relative_path));
                });
            });

            ui.add_space(16.0);
            ui.separator();
            ui.heading("Metadata");
            ui.add_space(8.0);

            egui::Grid::new("movie-detail-fields")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Title");
                    let mut title = movie.title.clone();
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut title));
                    ui.end_row();

                    ui.label("Year");
                    let mut year = movie
                        .year
                        .map(|y| y.to_string())
                        .unwrap_or_default();
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut year));
                    ui.end_row();

                    ui.label("TMDB id");
                    let mut tmdb_id = movie.tmdb_id.clone().unwrap_or_default();
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut tmdb_id));
                    ui.end_row();

                    ui.label("IMDb id");
                    let mut imdb_id = movie.imdb_id.clone().unwrap_or_default();
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut imdb_id));
                    ui.end_row();
                });

            ui.add_space(12.0);
            ui.label("Summary");
            let mut summary = movie.summary.clone().unwrap_or_default();
            ui.add_enabled(
                false,
                egui::TextEdit::multiline(&mut summary).desired_rows(6),
            );

            ui.add_space(16.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.add_enabled(false, egui::Button::new("Save changes"));
                ui.label("Editing and TMDB rematch coming next.");
            });
        });
    }

    fn poster(&self, ui: &mut Ui, movie: &MovieDetail) {
        let size = egui::vec2(POSTER_WIDTH, POSTER_WIDTH * 1.5);
        if let Some(url) = movie.poster_url.as_deref().filter(|url| !url.is_empty()) {
            ui.add(
                Image::new(url)
                    .fit_to_exact_size(size)
                    .corner_radius(6.0)
                    .show_loading_spinner(true),
            );
        } else {
            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
            ui.painter().rect_filled(rect, 6.0, ui.visuals().faint_bg_color);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                movie.title.chars().next().unwrap_or('?'),
                egui::FontId::proportional(32.0),
                ui.visuals().weak_text_color(),
            );
        }
    }
}
