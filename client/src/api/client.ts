import type {
  ApiErrorBody,
  BrowseResponse,
  FavoriteResponse,
  GenresResponse,
  LibraryStatusResponse,
  MovieDetail,
  MovieListResponse,
  MovieSummary,
  PersonDetail,
  ScanStreamEvent,
  SearchResponse,
} from './types';
import { readSseStream } from './sse';
import { normalizeMovieDetail } from './normalize';

export class LoonApiError extends Error {
  readonly code: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = 'LoonApiError';
    this.code = code;
  }
}

async function request<T>(
  baseUrl: string,
  path: string,
  init?: RequestInit,
): Promise<T> {
  const response = await fetch(`${baseUrl}${path}`, init);
  const text = await response.text();
  if (!response.ok) {
    try {
      const body = JSON.parse(text) as ApiErrorBody;
      throw new LoonApiError(body.error.code, body.error.message);
    } catch (error) {
      if (error instanceof LoonApiError) {
        throw error;
      }
      throw new LoonApiError('http_error', `HTTP ${response.status}`);
    }
  }
  if (!text) {
    throw new LoonApiError('empty_response', 'Empty response from server');
  }
  try {
    return JSON.parse(text) as T;
  } catch {
    throw new LoonApiError('invalid_json', 'Invalid JSON from server');
  }
}

export interface ListMoviesOptions {
  page?: number;
  limit?: number;
  sort?: 'title' | 'year' | 'recently_added';
  genre?: string;
}

export async function fetchMovies(
  baseUrl: string,
  options: ListMoviesOptions = {},
): Promise<MovieListResponse> {
  const params = new URLSearchParams();
  if (options.page) {
    params.set('page', String(options.page));
  }
  if (options.limit) {
    params.set('limit', String(options.limit));
  }
  if (options.sort) {
    params.set('sort', options.sort);
  }
  if (options.genre) {
    params.set('genre', options.genre);
  }
  const query = params.toString();
  return request<MovieListResponse>(baseUrl, `/api/movies${query ? `?${query}` : ''}`);
}

export async function fetchMovie(
  baseUrl: string,
  slug: string,
  options: { cacheBust?: number } = {},
): Promise<MovieDetail> {
  const suffix = options.cacheBust ? `?_=${options.cacheBust}` : '';
  const detail = await request<MovieDetail>(
    baseUrl,
    `/api/movies/${encodeURIComponent(slug)}${suffix}`,
    options.cacheBust ? { cache: 'no-store' } : undefined,
  );
  return normalizeMovieDetail(detail);
}

export async function setMovieTmdbMatch(
  baseUrl: string,
  slug: string,
  tmdbId: string,
): Promise<MovieDetail> {
  const detail = await request<MovieDetail>(
    baseUrl,
    `/api/movies/${encodeURIComponent(slug)}/match`,
    {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ tmdb_id: tmdbId }),
    },
  );
  return normalizeMovieDetail(detail);
}

export async function fetchPerson(baseUrl: string, tmdbPersonId: number): Promise<PersonDetail> {
  return request<PersonDetail>(
    baseUrl,
    `/api/people/${encodeURIComponent(String(tmdbPersonId))}`,
  );
}

export async function searchMovies(
  baseUrl: string,
  query: string,
  limit = 20,
): Promise<SearchResponse> {
  const params = new URLSearchParams({ q: query, limit: String(limit) });
  return request<SearchResponse>(baseUrl, `/api/search?${params}`);
}

export async function fetchGenres(baseUrl: string): Promise<GenresResponse> {
  return request<GenresResponse>(baseUrl, '/api/genres');
}

export async function fetchBrowse(baseUrl: string): Promise<BrowseResponse> {
  return request<BrowseResponse>(baseUrl, '/api/browse');
}

export async function fetchFavorites(baseUrl: string): Promise<MovieSummary[]> {
  const browse = await fetchBrowse(baseUrl);
  const row = browse.rows.find((entry) => entry.row_type === 'favorites');
  return row?.movies ?? [];
}

export async function setFavorite(
  baseUrl: string,
  slug: string,
  favorite?: boolean,
): Promise<FavoriteResponse> {
  const init: RequestInit = {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
  };
  if (favorite !== undefined) {
    init.body = JSON.stringify({ favorite });
  }
  return request<FavoriteResponse>(
    baseUrl,
    `/api/movies/${encodeURIComponent(slug)}/favorite`,
    init,
  );
}

export async function fetchSimilarMovies(
  baseUrl: string,
  detail: MovieDetail,
  limit = 12,
): Promise<MovieSummary[]> {
  const genre = detail.genres[0];
  if (!genre) {
    return [];
  }
  const list = await fetchMovies(baseUrl, { genre, limit: limit + 1, page: 1, sort: 'title' });
  return list.movies.filter((movie) => movie.slug !== detail.slug).slice(0, limit);
}

/** Fetch every page of the movie list, sorted by title on the server. */
export async function fetchAllMovies(
  baseUrl: string,
  options: Omit<ListMoviesOptions, 'page' | 'limit'> = {},
): Promise<MovieSummary[]> {
  const pageSize = 100;
  const first = await fetchMovies(baseUrl, {
    ...options,
    page: 1,
    limit: pageSize,
    sort: 'title',
  });

  const movies = [...first.movies];
  const pages = first.pages ?? 1;

  for (let page = 2; page <= pages; page += 1) {
    const next = await fetchMovies(baseUrl, {
      ...options,
      page,
      limit: pageSize,
      sort: 'title',
    });
    movies.push(...next.movies);
  }

  return movies.sort((a, b) =>
    a.title.localeCompare(b.title, undefined, { sensitivity: 'base' }),
  );
}

export async function fetchLibraryStatus(baseUrl: string): Promise<LibraryStatusResponse> {
  return request<LibraryStatusResponse>(baseUrl, '/api/library/status');
}

export interface StreamLibraryScanOptions {
  full?: boolean;
  signal?: AbortSignal;
}

/** POST /api/library/scan and stream Server-Sent Events until complete. */
export async function streamLibraryScan(
  baseUrl: string,
  options: StreamLibraryScanOptions,
  onEvent: (event: ScanStreamEvent) => void,
): Promise<void> {
  const response = await fetch(`${baseUrl}/api/library/scan`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Accept: 'text/event-stream',
    },
    body: JSON.stringify({ full: options.full ?? false }),
    signal: options.signal,
  });

  if (!response.ok) {
    const text = await response.text();
    try {
      const body = JSON.parse(text) as ApiErrorBody;
      throw new LoonApiError(body.error.code, body.error.message);
    } catch (error) {
      if (error instanceof LoonApiError) {
        throw error;
      }
      throw new LoonApiError('http_error', `HTTP ${response.status}`);
    }
  }

  await readSseStream(
    response,
    (message) => {
      const event = JSON.parse(message.data) as ScanStreamEvent;
      onEvent(event);
    },
    options.signal,
  );
}
