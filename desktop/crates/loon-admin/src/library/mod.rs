//! Library browse: movie list and detail navigation.

mod movie_detail;
mod movies_table;

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use egui::Ui;
use nest_core::AppContext;
use nest_error::NestResult;

use crate::api::{LoonApiClient, MovieDetail, MovieSummary};

pub use movie_detail::MovieDetailPanel;
pub use movies_table::MoviesTable;

/// Library sub-routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryRoute {
    /// Movie table list.
    List,
    /// Single movie detail / edit screen.
    Detail {
        /// Movie slug being viewed.
        slug: String,
    },
}

/// Async fetch state shared by list and detail panels.
#[derive(Default)]
enum LoadState<T> {
    #[default]
    Idle,
    Loading,
    Ready(T),
    Failed(String),
}

/// Library section state and server fetches.
pub struct LibraryPanel {
    route: LibraryRoute,
    movies: LoadState<Vec<MovieSummary>>,
    movies_rx: Option<Receiver<Result<Vec<MovieSummary>, String>>>,
    detail: LoadState<MovieDetail>,
    detail_rx: Option<Receiver<Result<MovieDetail, String>>>,
    detail_slug: Option<String>,
    search: String,
    image_loaders_installed: bool,
    table: MoviesTable,
    detail_panel: MovieDetailPanel,
}

impl Default for LibraryPanel {
    fn default() -> Self {
        Self {
            route: LibraryRoute::List,
            movies: LoadState::Idle,
            movies_rx: None,
            detail: LoadState::Idle,
            detail_rx: None,
            detail_slug: None,
            search: String::new(),
            image_loaders_installed: false,
            table: MoviesTable::default(),
            detail_panel: MovieDetailPanel::default(),
        }
    }
}

impl LibraryPanel {
    /// Renders the library route (list or detail).
    pub fn ui(&mut self, ui: &mut Ui, ctx: &AppContext, server_url: &str) -> NestResult<()> {
        self.ensure_image_loaders(ui);

        match self.route.clone() {
            LibraryRoute::List => {
                self.poll_movies();
                self.list_ui(ui, ctx, server_url)?;
            }
            LibraryRoute::Detail { slug } => {
                self.poll_detail();
                self.detail_ui(ui, ctx, server_url, &slug)?;
            }
        }

        Ok(())
    }

    fn list_ui(&mut self, ui: &mut Ui, ctx: &AppContext, server_url: &str) -> NestResult<()> {
        ui.horizontal(|ui| {
            ui.heading("Movies");
            if ui.button("Refresh").clicked() {
                self.start_movies_fetch(server_url);
            }
            ui.separator();
            ui.label(format!("Server: {server_url}"));
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Search");
            ui.text_edit_singleline(&mut self.search);
        });
        ui.add_space(8.0);

        match &self.movies {
            LoadState::Idle => {
                ui.label("Loading movies…");
                self.start_movies_fetch(server_url);
            }
            LoadState::Loading => {
                ui.spinner();
                ui.label("Loading movies…");
            }
            LoadState::Failed(message) => {
                ui.colored_label(ui.visuals().error_fg_color, message);
                if ui.button("Retry").clicked() {
                    self.start_movies_fetch(server_url);
                }
            }
            LoadState::Ready(movies) => {
                let filtered = filter_movies(movies, &self.search);
                ui.label(format!("{} movies", filtered.len()));
                ui.add_space(4.0);

                let selected = self.table.show(ui, &filtered);
                if let Some(slug) = selected {
                    self.open_detail(slug);
                }
            }
        }

        let _ = ctx;
        Ok(())
    }

    fn detail_ui(
        &mut self,
        ui: &mut Ui,
        ctx: &AppContext,
        server_url: &str,
        slug: &str,
    ) -> NestResult<()> {
        if self.detail_slug.as_deref() != Some(slug) {
            self.open_detail(slug.to_string());
        }

        ui.horizontal(|ui| {
            if ui.button("← Back to list").clicked() {
                self.route = LibraryRoute::List;
                self.detail = LoadState::Idle;
                self.detail_rx = None;
                self.detail_slug = None;
            }
        });
        ui.add_space(8.0);

        match &self.detail {
            LoadState::Idle => {
                ui.spinner();
                ui.label("Loading movie…");
                self.start_detail_fetch(server_url, slug);
            }
            LoadState::Loading => {
                ui.spinner();
                ui.label("Loading movie…");
            }
            LoadState::Failed(message) => {
                ui.colored_label(ui.visuals().error_fg_color, message);
                if ui.button("Retry").clicked() {
                    self.start_detail_fetch(server_url, slug);
                }
            }
            LoadState::Ready(movie) => {
                self.detail_panel.show(ui, movie);
            }
        }

        let _ = ctx;
        Ok(())
    }

    fn open_detail(&mut self, slug: String) {
        self.route = LibraryRoute::Detail { slug: slug.clone() };
        self.detail = LoadState::Idle;
        self.detail_rx = None;
        self.detail_slug = Some(slug);
    }

    fn start_movies_fetch(&mut self, server_url: &str) {
        let (tx, rx) = mpsc::channel();
        self.movies_rx = Some(rx);
        self.movies = LoadState::Loading;

        let server_url = server_url.to_string();
        thread::spawn(move || {
            let result = fetch_movies(&server_url).map_err(|error| error.to_string());
            let _ = tx.send(result);
        });
    }

    fn start_detail_fetch(&mut self, server_url: &str, slug: &str) {
        let (tx, rx) = mpsc::channel();
        self.detail_rx = Some(rx);
        self.detail = LoadState::Loading;

        let server_url = server_url.to_string();
        let slug = slug.to_string();
        thread::spawn(move || {
            let result = fetch_movie(&server_url, &slug).map_err(|error| error.to_string());
            let _ = tx.send(result);
        });
    }

    fn poll_movies(&mut self) {
        let Some(rx) = self.movies_rx.as_ref() else {
            return;
        };

        match rx.try_recv() {
            Ok(Ok(movies)) => {
                self.movies = LoadState::Ready(movies);
                self.movies_rx = None;
            }
            Ok(Err(message)) => {
                self.movies = LoadState::Failed(message);
                self.movies_rx = None;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.movies = LoadState::Failed("movie fetch thread stopped".into());
                self.movies_rx = None;
            }
        }
    }

    fn poll_detail(&mut self) {
        let Some(rx) = self.detail_rx.as_ref() else {
            return;
        };

        match rx.try_recv() {
            Ok(Ok(movie)) => {
                self.detail = LoadState::Ready(movie);
                self.detail_rx = None;
            }
            Ok(Err(message)) => {
                self.detail = LoadState::Failed(message);
                self.detail_rx = None;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.detail = LoadState::Failed("movie detail fetch thread stopped".into());
                self.detail_rx = None;
            }
        }
    }

    fn ensure_image_loaders(&mut self, ui: &mut Ui) {
        if !self.image_loaders_installed {
            egui_extras::install_image_loaders(ui.ctx());
            self.image_loaders_installed = true;
        }
    }
}

fn filter_movies(movies: &[MovieSummary], search: &str) -> Vec<MovieSummary> {
    let query = search.trim().to_ascii_lowercase();
    if query.is_empty() {
        return movies.to_vec();
    }

    movies
        .iter()
        .filter(|movie| {
            movie.title.to_ascii_lowercase().contains(&query)
                || movie
                    .summary
                    .to_ascii_lowercase()
                    .contains(&query)
                || movie
                    .slug
                    .to_ascii_lowercase()
                    .contains(&query)
        })
        .cloned()
        .collect()
}

fn fetch_movies(server_url: &str) -> NestResult<Vec<MovieSummary>> {
    let client = LoonApiClient::new(server_url)?;
    Ok(client.list_movies()?.movies)
}

fn fetch_movie(server_url: &str, slug: &str) -> NestResult<MovieDetail> {
    let client = LoonApiClient::new(server_url)?;
    client.get_movie(slug)
}
