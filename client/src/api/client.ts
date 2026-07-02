import type { ApiErrorBody, MovieListResponse, MovieSummary } from './types';

export class LoonApiError extends Error {
  readonly code: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = 'LoonApiError';
    this.code = code;
  }
}

async function request<T>(baseUrl: string, path: string): Promise<T> {
  const response = await fetch(`${baseUrl}${path}`);
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

export async function fetchMovies(baseUrl: string): Promise<MovieSummary[]> {
  const body = await request<MovieListResponse>(baseUrl, '/api/movies');
  return body.movies;
}
