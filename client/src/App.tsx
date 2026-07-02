import { useCallback, useEffect, useState } from 'react';
import { fetchMovies } from './api/client';
import type { MovieSummary } from './api/types';
import { ContentRow } from './components/ContentRow';
import { getServerUrl, resolveArtworkUrl, streamUrl } from './config';
import { VideoPlayer } from './player/VideoPlayer';
import styles from './App.module.css';

type Screen =
  | { kind: 'browse' }
  | { kind: 'player'; movie: MovieSummary };

export function App() {
  const server = getServerUrl();
  const [screen, setScreen] = useState<Screen>({ kind: 'browse' });
  const [movies, setMovies] = useState<MovieSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadMovies = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await fetchMovies(server);
      setMovies(list);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to load movies';
      setError(message);
    } finally {
      setLoading(false);
    }
  }, [server]);

  useEffect(() => {
    void loadMovies();
  }, [loadMovies]);

  const resolveArtwork = useCallback(
    (path: string | undefined) => resolveArtworkUrl(path, server),
    [server],
  );

  if (screen.kind === 'player') {
    return (
      <VideoPlayer
        src={streamUrl(server, screen.movie.slug)}
        title={screen.movie.title}
        onBack={() => setScreen({ kind: 'browse' })}
      />
    );
  }

  return (
    <main className={styles.shell}>
      <header className={styles.header}>
        <span className={styles.logo}>Loon</span>
        <span className={styles.hint}>Arrow keys + Enter · Back to exit player</span>
      </header>

      {loading ? <p className={styles.status}>Loading movies…</p> : null}
      {error ? (
        <div className={styles.error}>
          <p>{error}</p>
          <button type="button" onClick={() => void loadMovies()}>
            Retry
          </button>
        </div>
      ) : null}

      {!loading && !error ? (
        <ContentRow
          title="Movies"
          movies={movies}
          resolveArtwork={resolveArtwork}
          onSelect={(movie) => setScreen({ kind: 'player', movie })}
        />
      ) : null}
    </main>
  );
}
