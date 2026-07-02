import { useEffect, useRef, type RefObject } from 'react';
import {
  FocusContext,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import type { MovieSummary } from '../api/types';
import { PosterCard } from './PosterCard';
import styles from './ContentRow.module.css';

function posterFocusKey(slug: string): string {
  return `poster-${slug}`;
}

interface FocusablePosterProps {
  movie: MovieSummary;
  posterUrl?: string;
  onSelect: (movie: MovieSummary) => void;
  rowRef: RefObject<HTMLDivElement>;
}

function FocusablePoster({ movie, posterUrl, onSelect, rowRef }: FocusablePosterProps) {
  const { ref, focused } = useFocusable({
    focusKey: posterFocusKey(movie.slug),
    onEnterPress: () => onSelect(movie),
    onFocus: (layout) => {
      const row = rowRef.current;
      if (!row) {
        return;
      }
      const itemLeft = layout.x;
      const itemRight = layout.x + layout.width;
      const viewLeft = row.scrollLeft;
      const viewRight = viewLeft + row.clientWidth;
      if (itemLeft < viewLeft) {
        row.scrollLeft = itemLeft - 48;
      } else if (itemRight > viewRight) {
        row.scrollLeft = itemRight - row.clientWidth + 48;
      }
    },
  });

  return (
    <div ref={ref} className={styles.item}>
      <PosterCard movie={movie} posterUrl={posterUrl} focused={focused} />
    </div>
  );
}

interface ContentRowProps {
  title: string;
  movies: MovieSummary[];
  resolveArtwork: (path: string | undefined) => string | undefined;
  onSelect: (movie: MovieSummary) => void;
}

export function ContentRow({
  title,
  movies,
  resolveArtwork,
  onSelect,
}: ContentRowProps) {
  const rowRef = useRef<HTMLDivElement>(null);
  const firstFocusKey = movies[0] ? posterFocusKey(movies[0].slug) : undefined;

  const { ref, focusKey, focusSelf } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'movies-row',
    preferredChildFocusKey: firstFocusKey,
  });

  useEffect(() => {
    if (movies.length > 0) {
      focusSelf();
    }
  }, [movies, focusSelf]);

  return (
    <section className={styles.rowSection}>
      <h1 className={styles.rowTitle}>{title}</h1>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.rowScroller}>
          <div ref={rowRef} className={styles.row}>
            {movies.map((movie) => (
              <FocusablePoster
                key={movie.slug}
                movie={movie}
                posterUrl={resolveArtwork(movie.poster_url)}
                onSelect={onSelect}
                rowRef={rowRef}
              />
            ))}
          </div>
        </div>
      </FocusContext.Provider>
    </section>
  );
}
