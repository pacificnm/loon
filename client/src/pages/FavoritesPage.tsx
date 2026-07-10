import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { fetchFavorites } from '../api/client';
import type { MovieSummary } from '../api/types';
import { MovieVerticalList } from '../components/MovieVerticalList';
import { useServerUrl } from '../config';
import styles from './page.module.css';

interface FavoritesPageProps {
  focusEpoch?: number;
}

export function FavoritesPage({ focusEpoch }: FavoritesPageProps) {
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
      setMovies(await fetchFavorites(server));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load favorites');
    } finally {
      setLoading(false);
    }
  }, [server]);

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <div className={styles.page}>
      <h1 className={styles.heading}>Favorites</h1>
      {loading ? <p className={styles.status}>Loading favorites…</p> : null}
      {error ? <p className={styles.errorText}>{error}</p> : null}
      {!loading && !error && server ? (
        <MovieVerticalList
          movies={movies}
          server={server}
          focusEpoch={focusEpoch}
          onSelect={(movie) => navigate(`/movie/${movie.slug}`)}
        />
      ) : null}
    </div>
  );
}
