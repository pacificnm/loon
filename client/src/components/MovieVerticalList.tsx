import { useEffect, useRef, type RefObject } from 'react';
import {
  FocusContext,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import type { MovieSummary } from '../api/types';
import { resolveArtworkUrl } from '../config';
import styles from './MovieVerticalList.module.css';

function itemFocusKey(slug: string): string {
  return `movie-${slug}`;
}

interface MovieRowProps {
  movie: MovieSummary;
  server: string;
  listRef: RefObject<HTMLDivElement>;
  onSelect: (movie: MovieSummary) => void;
}

function MovieRow({ movie, server, listRef, onSelect }: MovieRowProps) {
  const posterUrl = resolveArtworkUrl(movie.poster_url, server);

  const { ref, focused } = useFocusable({
    focusKey: itemFocusKey(movie.slug),
    onEnterPress: () => onSelect(movie),
    onFocus: (layout) => {
      const list = listRef.current;
      if (!list) {
        return;
      }
      const itemTop = layout.y;
      const itemBottom = layout.y + layout.height;
      const viewTop = list.scrollTop;
      const viewBottom = viewTop + list.clientHeight;
      if (itemTop < viewTop) {
        list.scrollTop = itemTop - 16;
      } else if (itemBottom > viewBottom) {
        list.scrollTop = itemBottom - list.clientHeight + 16;
      }
    },
  });

  return (
    <article ref={ref} className={`${styles.row} ${focused ? styles.focused : ''}`}>
      <div className={styles.posterFrame}>
        {posterUrl ? (
          <img className={styles.poster} src={posterUrl} alt="" />
        ) : (
          <div className={styles.placeholder}>{movie.title.slice(0, 1)}</div>
        )}
      </div>
      <div className={styles.meta}>
        <h2 className={styles.title}>{movie.title}</h2>
        <p className={styles.subtitle}>
          {[movie.year, movie.runtime_minutes ? `${movie.runtime_minutes} min` : null]
            .filter(Boolean)
            .join(' · ')}
        </p>
        {movie.summary ? <p className={styles.summary}>{movie.summary}</p> : null}
      </div>
    </article>
  );
}

interface MovieVerticalListProps {
  movies: MovieSummary[];
  server: string;
  focusEpoch?: number;
  onSelect: (movie: MovieSummary) => void;
}

export function MovieVerticalList({
  movies,
  server,
  focusEpoch = 0,
  onSelect,
}: MovieVerticalListProps) {
  const listRef = useRef<HTMLDivElement>(null);
  const firstKey = movies[0] ? itemFocusKey(movies[0].slug) : undefined;

  const { ref, focusKey, focusSelf } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'movie-vertical-list',
    preferredChildFocusKey: firstKey,
  });

  useEffect(() => {
    if (movies.length > 0) {
      focusSelf();
    }
  }, [movies, focusSelf, focusEpoch]);

  if (movies.length === 0) {
    return <p className={styles.empty}>No movies found.</p>;
  }

  return (
    <FocusContext.Provider value={focusKey}>
      <div ref={ref} className={styles.wrapper}>
        <div ref={listRef} className={styles.list}>
          {movies.map((movie) => (
            <MovieRow
              key={movie.slug}
              movie={movie}
              server={server}
              listRef={listRef}
              onSelect={onSelect}
            />
          ))}
        </div>
      </div>
    </FocusContext.Provider>
  );
}
