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
  profile_url?: string;
}

export interface CrewMember {
  name: string;
  job?: string;
}

export interface MovieFileInfo {
  filename: string;
  relative_path: string;
  extension?: string | null;
  size_bytes?: number | null;
  content_type: string;
  modified_at?: number | null;
  scanned_at?: number | null;
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
  tmdb_id?: string | null;
  imdb_id?: string | null;
  file?: MovieFileInfo | null;
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

export type ScanPhase = 'discovering' | 'enriching' | 'persisting';

export interface ScanProgress {
  phase?: ScanPhase | null;
  files_seen: number;
  candidates: number;
  errors: number;
  enriched: number;
  total_to_enrich: number;
  current_path?: string | null;
}

export interface ScanStats {
  files_seen: number;
  candidates: number;
  errors: number;
}

export type ScanStreamEvent =
  | { type: 'started'; scan_id: string }
  | { type: 'progress'; progress: ScanProgress }
  | {
      type: 'complete';
      scan_id: string;
      movies_count: number;
      duration_secs: number;
      stats: ScanStats;
    }
  | { type: 'error'; scan_id: string; message: string };

export interface LibraryStatusResponse {
  state: string;
  last_scan_at?: string | null;
  last_scan_duration_secs: number;
  movies_count: number;
  scan_in_progress: boolean;
  progress?: ScanProgress | null;
}
