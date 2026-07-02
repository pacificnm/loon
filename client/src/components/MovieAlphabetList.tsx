import { useEffect, useMemo, useRef, type RefObject } from 'react';
import {
  FocusContext,
  setFocus,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import type { MovieSummary } from '../api/types';
import { resolveArtworkUrl } from '../config';
import {
  ALPHABET_LETTERS,
  groupMoviesByLetter,
  type LetterGroup,
} from '../utils/alphabet';
import styles from './MovieAlphabetList.module.css';

function itemFocusKey(slug: string): string {
  return `movie-${slug}`;
}

function sectionFocusKey(letter: string): string {
  return `section-${letter}`;
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
        <h3 className={styles.movieTitle}>{movie.title}</h3>
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

interface LetterSectionProps {
  group: LetterGroup;
  server: string;
  listRef: RefObject<HTMLDivElement>;
  onSelect: (movie: MovieSummary) => void;
}

function LetterSection({ group, server, listRef, onSelect }: LetterSectionProps) {
  const { ref, focusKey } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: sectionFocusKey(group.letter),
    preferredChildFocusKey: itemFocusKey(group.movies[0]?.slug ?? ''),
  });

  return (
    <section ref={ref} className={styles.section} data-letter={group.letter}>
      <h2 className={styles.letterHeading}>{group.letter}</h2>
      <FocusContext.Provider value={focusKey}>
        <div className={styles.sectionMovies}>
          {group.movies.map((movie) => (
            <MovieRow
              key={movie.slug}
              movie={movie}
              server={server}
              listRef={listRef}
              onSelect={onSelect}
            />
          ))}
        </div>
      </FocusContext.Provider>
    </section>
  );
}

interface AlphabetIndexProps {
  activeLetters: Set<string>;
  onJump: (letter: string) => void;
}

function AlphabetIndex({ activeLetters, onJump }: AlphabetIndexProps) {
  const { ref, focusKey } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'alphabet-index',
  });

  return (
    <aside className={styles.indexRail}>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.indexLetters}>
          {ALPHABET_LETTERS.map((letter) => {
            const enabled = activeLetters.has(letter);
            return (
              <IndexLetter
                key={letter}
                letter={letter}
                enabled={enabled}
                onJump={onJump}
              />
            );
          })}
        </div>
      </FocusContext.Provider>
    </aside>
  );
}

function IndexLetter({
  letter,
  enabled,
  onJump,
}: {
  letter: string;
  enabled: boolean;
  onJump: (letter: string) => void;
}) {
  const { ref, focused } = useFocusable({
    focusKey: `index-${letter}`,
    focusable: enabled,
    onEnterPress: () => onJump(letter),
  });

  const className = [
    styles.indexLetter,
    enabled ? styles.indexEnabled : styles.indexDisabled,
    focused ? styles.indexFocused : '',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <button
      ref={ref}
      type="button"
      className={className}
      disabled={!enabled}
      onClick={() => enabled && onJump(letter)}
    >
      {letter}
    </button>
  );
}

interface MovieAlphabetListProps {
  movies: MovieSummary[];
  server: string;
  focusEpoch?: number;
  onSelect: (movie: MovieSummary) => void;
}

export function MovieAlphabetList({
  movies,
  server,
  focusEpoch = 0,
  onSelect,
}: MovieAlphabetListProps) {
  const listRef = useRef<HTMLDivElement>(null);
  const groups = useMemo(() => groupMoviesByLetter(movies), [movies]);
  const activeLetters = useMemo(() => new Set(groups.map((g) => g.letter)), [groups]);
  const firstMovie = groups[0]?.movies[0];

  const { ref, focusKey, focusSelf } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'movie-alphabet-list',
    preferredChildFocusKey: firstMovie ? itemFocusKey(firstMovie.slug) : undefined,
  });

  useEffect(() => {
    if (firstMovie) {
      focusSelf();
    }
  }, [firstMovie, focusSelf, focusEpoch]);

  const scrollToLetter = (letter: string) => {
    const list = listRef.current;
    if (!list) {
      return;
    }
    const section = list.querySelector<HTMLElement>(`[data-letter="${letter}"]`);
    if (!section) {
      return;
    }
    list.scrollTop = section.offsetTop;
    const first = groups.find((group) => group.letter === letter)?.movies[0];
    if (first) {
      setFocus(itemFocusKey(first.slug));
    }
  };

  if (movies.length === 0) {
    return <p className={styles.empty}>No movies found.</p>;
  }

  return (
    <div className={styles.shell}>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.listPane}>
          <div ref={listRef} className={styles.list}>
            {groups.map((group) => (
              <LetterSection
                key={group.letter}
                group={group}
                server={server}
                listRef={listRef}
                onSelect={onSelect}
              />
            ))}
          </div>
        </div>
      </FocusContext.Provider>
      <AlphabetIndex activeLetters={activeLetters} onJump={scrollToLetter} />
    </div>
  );
}
