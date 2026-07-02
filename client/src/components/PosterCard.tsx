import type { MovieSummary } from '../api/types';
import styles from './PosterCard.module.css';

interface PosterCardProps {
  movie: MovieSummary;
  posterUrl?: string;
  focused: boolean;
}

export function PosterCard({ movie, posterUrl, focused }: PosterCardProps) {
  return (
    <article className={`${styles.card} ${focused ? styles.focused : ''}`}>
      <div className={styles.posterFrame}>
        {posterUrl ? (
          <img className={styles.poster} src={posterUrl} alt="" loading="lazy" />
        ) : (
          <div className={styles.placeholder}>{movie.title.slice(0, 1)}</div>
        )}
      </div>
      <h2 className={styles.title}>{movie.title}</h2>
      {movie.year ? <p className={styles.year}>{movie.year}</p> : null}
    </article>
  );
}
