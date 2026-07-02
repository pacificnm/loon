import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  FocusContext,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import { searchMovies } from '../api/client';
import type { MovieSummary } from '../api/types';
import { MovieVerticalList } from '../components/MovieVerticalList';
import { getServerUrl } from '../config';
import styles from './page.module.css';

interface SearchPageProps {
  focusEpoch?: number;
}

export function SearchPage({ focusEpoch }: SearchPageProps) {
  const server = getServerUrl();
  const navigate = useNavigate();
  const [query, setQuery] = useState('');
  const [movies, setMovies] = useState<MovieSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const { ref, focusKey, focusSelf } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'search-page',
    preferredChildFocusKey: 'search-input',
  });

  const runSearch = useCallback(
    async (text: string) => {
      const trimmed = text.trim();
      if (trimmed.length < 2) {
        setMovies([]);
        setError(null);
        return;
      }
      setLoading(true);
      setError(null);
      try {
        const response = await searchMovies(server, trimmed);
        setMovies(response.movies);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Search failed');
        setMovies([]);
      } finally {
        setLoading(false);
      }
    },
    [server],
  );

  useEffect(() => {
    focusSelf();
  }, [focusSelf, focusEpoch]);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void runSearch(query);
    }, 300);
    return () => window.clearTimeout(timer);
  }, [query, runSearch]);

  return (
    <div className={styles.page}>
      <h1 className={styles.heading}>Search</h1>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.searchBox}>
          <SearchInput value={query} onChange={setQuery} />
        </div>
      </FocusContext.Provider>
      {loading ? <p className={styles.status}>Searching…</p> : null}
      {error ? <p className={styles.errorText}>{error}</p> : null}
      {!loading && !error && query.trim().length >= 2 ? (
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

function SearchInput({
  value,
  onChange,
}: {
  value: string;
  onChange: (value: string) => void;
}) {
  const { ref, focused } = useFocusable({
    focusKey: 'search-input',
  });

  return (
    <input
      ref={ref}
      className={`${styles.searchInput} ${focused ? styles.searchInputFocused : ''}`}
      type="text"
      placeholder="Search movies…"
      value={value}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}
