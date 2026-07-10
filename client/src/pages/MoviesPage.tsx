import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { fetchAllMovies } from '../api/client';
import type { MovieSummary } from '../api/types';
import { MovieAlphabetList } from '../components/MovieAlphabetList';
import { useServerUrl } from '../config';
import styles from './page.module.css';

interface MoviesPageProps {
  focusEpoch?: number;
  genre?: string;
}

export function MoviesPage({ focusEpoch, genre }: MoviesPageProps) {
  const server = useServerUrl();
  const navigate = useNavigate();
  const [movies, setMovies] = useState<MovieSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    if (!server) {
      setLoading(false);
      setMovies([]);
      setError('No server configured. Open Admin → Settings.');
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const list = await fetchAllMovies(server, { genre });
      setMovies(list);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load movies');
    } finally {
      setLoading(false);
    }
  }, [server, genre]);

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <div className={styles.page}>
      <h1 className={styles.heading}>{genre ? genre : 'Movies'}</h1>
      <div className={styles.content}>
        {loading ? <p className={styles.status}>Loading movies…</p> : null}
        {error ? (
          <div className={styles.error}>
            <p>{error}</p>
            <button type="button" onClick={() => void load()}>
              Retry
            </button>
          </div>
        ) : null}
        {!loading && !error && server ? (
          <MovieAlphabetList
            movies={movies}
            server={server}
            focusEpoch={focusEpoch}
            onSelect={(movie) => navigate(`/movie/${movie.slug}`)}
          />
        ) : null}
      </div>
    </div>
  );
}
