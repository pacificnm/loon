import {
  FocusContext,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import type { MovieSummary } from '../api/types';
import { PosterCard } from './PosterCard';
import styles from './ContentRow.module.css';

interface FocusablePosterProps {
  movie: MovieSummary;
  posterUrl?: string;
  onSelect: (movie: MovieSummary) => void;
}

function FocusablePoster({ movie, posterUrl, onSelect }: FocusablePosterProps) {
  const { ref, focused } = useFocusable({
    onEnterPress: () => onSelect(movie),
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
  const { ref, focusKey } = useFocusable({
    focusable: false,
    trackChildren: true,
  });

  return (
    <section className={styles.rowSection}>
      <h1 className={styles.rowTitle}>{title}</h1>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.row}>
          {movies.map((movie) => (
            <FocusablePoster
              key={movie.slug}
              movie={movie}
              posterUrl={resolveArtwork(movie.poster_url)}
              onSelect={onSelect}
            />
          ))}
        </div>
      </FocusContext.Provider>
    </section>
  );
}
