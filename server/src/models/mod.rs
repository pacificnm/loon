//! API response models.

mod browse;
mod library;
mod movie;
mod root;

pub use browse::{BrowseResponse, BrowseRow};
pub use library::{
    FavoriteRequest, FavoriteResponse, GenreEntry, GenresResponse, LibraryStatusResponse,
    ProgressRequest, ProgressResponse, ScanStartRequest, SearchResponse,
};
pub use movie::{
    CastMemberDto, CrewMemberDto, HealthResponse, MovieDetail, MovieListResponse, MovieSummary,
};
pub use root::RootResponse;
