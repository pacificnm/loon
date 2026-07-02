export interface MovieSummary {
  slug: string;
  title: string;
  year?: number;
  runtime_minutes: number;
  poster_url?: string;
  backdrop_url?: string;
  summary: string;
}

export interface MovieListResponse {
  movies: MovieSummary[];
  total: number;
  page?: number;
  limit?: number;
  pages?: number;
}

export interface CastMember {
  name: string;
  character?: string;
}

export interface CrewMember {
  name: string;
  job?: string;
}

export interface MovieDetail {
  slug: string;
  title: string;
  original_title?: string;
  year?: number;
  runtime_minutes?: number;
  summary?: string;
  genres: string[];
  poster_url?: string;
  backdrop_url?: string;
  cast: CastMember[];
  crew: CrewMember[];
  is_favorite: boolean;
  watch_progress_seconds?: number;
  stream_url: string;
}

export interface SearchResponse {
  query: string;
  movies: MovieSummary[];
  total: number;
}

export interface GenreEntry {
  name: string;
  count: number;
}

export interface GenresResponse {
  genres: GenreEntry[];
}

export interface FavoriteResponse {
  slug: string;
  favorite: boolean;
}

export interface BrowseRow {
  slug: string;
  title: string;
  row_type: string;
  movies: MovieSummary[];
}

export interface BrowseResponse {
  hero?: MovieSummary;
  rows: BrowseRow[];
}

export interface ApiErrorBody {
  error: {
    code: string;
    message: string;
  };
}
