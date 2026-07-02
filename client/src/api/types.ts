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
  total?: number;
}

export interface ApiErrorBody {
  error: {
    code: string;
    message: string;
  };
}
