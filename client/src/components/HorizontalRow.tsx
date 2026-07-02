import { useRef, type RefObject } from 'react';
import {
  FocusContext,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import type { MovieSummary } from '../api/types';
import { PosterCard } from './PosterCard';
import styles from './HorizontalRow.module.css';

function cardFocusKey(prefix: string, slug: string): string {
  return `${prefix}-${slug}`;
}

interface FocusableCardProps {
  prefix: string;
  movie: MovieSummary;
  posterUrl?: string;
  rowRef: RefObject<HTMLDivElement>;
  onSelect: (movie: MovieSummary) => void;
}

function FocusableCard({ prefix, movie, posterUrl, rowRef, onSelect }: FocusableCardProps) {
  const select = () => onSelect(movie);

  const { ref, focused } = useFocusable({
    focusKey: cardFocusKey(prefix, movie.slug),
    onEnterPress: select,
    onFocus: (layout) => {
      const row = rowRef.current;
      if (!row) {
        return;
      }
      const rowRect = row.getBoundingClientRect();
      const itemLeft = layout.x;
      const itemRight = layout.x + layout.width;
      if (itemLeft < rowRect.left + 8) {
        row.scrollLeft -= rowRect.left - itemLeft + 48;
      } else if (itemRight > rowRect.right - 8) {
        row.scrollLeft += itemRight - rowRect.right + 48;
      }
    },
  });

  return (
    <div
      ref={ref}
      className={styles.item}
      role="button"
      tabIndex={-1}
      onClick={select}
    >
      <PosterCard movie={movie} posterUrl={posterUrl} focused={focused} />
    </div>
  );
}

interface HorizontalRowProps {
  title: string;
  prefix: string;
  movies: MovieSummary[];
  resolveArtwork: (path: string | undefined) => string | undefined;
  onSelect: (movie: MovieSummary) => void;
}

export function HorizontalRow({
  title,
  prefix,
  movies,
  resolveArtwork,
  onSelect,
}: HorizontalRowProps) {
  const rowRef = useRef<HTMLDivElement>(null);
  const firstKey = movies[0] ? cardFocusKey(prefix, movies[0].slug) : undefined;

  const { ref, focusKey } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: `row-${prefix}`,
    preferredChildFocusKey: firstKey,
  });

  if (movies.length === 0) {
    return null;
  }

  return (
    <section className={styles.section}>
      <h2 className={styles.title}>{title}</h2>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.scroller}>
          <div ref={rowRef} className={styles.row}>
            {movies.map((movie) => (
              <FocusableCard
                key={movie.slug}
                prefix={prefix}
                movie={movie}
                posterUrl={resolveArtwork(movie.poster_url)}
                rowRef={rowRef}
                onSelect={onSelect}
              />
            ))}
          </div>
        </div>
      </FocusContext.Provider>
    </section>
  );
}
