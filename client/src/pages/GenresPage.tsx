import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  FocusContext,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import { fetchGenres } from '../api/client';
import type { GenreEntry } from '../api/types';
import { getServerUrl } from '../config';
import styles from './page.module.css';

interface GenresPageProps {
  focusEpoch?: number;
}

function GenreItem({
  genre,
  onSelect,
}: {
  genre: GenreEntry;
  onSelect: (name: string) => void;
}) {
  const { ref, focused } = useFocusable({
    focusKey: `genre-${genre.name}`,
    onEnterPress: () => onSelect(genre.name),
  });

  return (
    <button
      ref={ref}
      type="button"
      className={`${styles.genreItem} ${focused ? styles.genreFocused : ''}`}
      onClick={() => onSelect(genre.name)}
    >
      <span>{genre.name}</span>
      <span className={styles.genreCount}>{genre.count}</span>
    </button>
  );
}

export function GenresPage({ focusEpoch }: GenresPageProps) {
  const server = getServerUrl();
  const navigate = useNavigate();
  const [genres, setGenres] = useState<GenreEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const { ref, focusKey, focusSelf } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'genres-page',
    preferredChildFocusKey: genres[0] ? `genre-${genres[0].name}` : undefined,
  });

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetchGenres(server);
      setGenres(response.genres);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load genres');
    } finally {
      setLoading(false);
    }
  }, [server]);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    if (genres.length > 0) {
      focusSelf();
    }
  }, [genres, focusSelf, focusEpoch]);

  return (
    <div className={styles.page}>
      <h1 className={styles.heading}>Genres</h1>
      {loading ? <p className={styles.status}>Loading genres…</p> : null}
      {error ? <p className={styles.errorText}>{error}</p> : null}
      {!loading && !error ? (
        <FocusContext.Provider value={focusKey}>
          <div ref={ref} className={styles.genreList}>
            {genres.map((genre) => (
              <GenreItem
                key={genre.name}
                genre={genre}
                onSelect={(name) => navigate(`/genre/${encodeURIComponent(name)}`)}
              />
            ))}
          </div>
        </FocusContext.Provider>
      ) : null}
    </div>
  );
}
