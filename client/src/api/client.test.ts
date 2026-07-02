import { afterEach, describe, expect, it, vi } from 'vitest';
import { fetchMovies } from './client';

describe('fetchMovies', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('returns movies from list response', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        text: async () =>
          JSON.stringify({
            movies: [
              {
                slug: 'alien-1979',
                title: 'Alien',
                year: 1979,
                runtime_minutes: 117,
                summary: 'In space no one can hear you scream.',
              },
            ],
          }),
      }),
    );

    const movies = await fetchMovies('http://localhost:3000');
    expect(movies).toHaveLength(1);
    expect(movies[0]?.slug).toBe('alien-1979');
  });

  it('throws LoonApiError on server error envelope', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: false,
        status: 404,
        text: async () =>
          JSON.stringify({
            error: { code: 'movie_not_found', message: 'missing' },
          }),
      }),
    );

    await expect(fetchMovies('http://localhost:3000')).rejects.toMatchObject({
      code: 'movie_not_found',
    });
  });
});
