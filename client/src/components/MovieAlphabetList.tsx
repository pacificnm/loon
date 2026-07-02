import { useEffect, useMemo, useRef, type RefObject } from 'react';
import {
  FocusContext,
  setFocus,
  updateAllLayouts,
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

function scrollItemIntoView(list: HTMLElement, layout: { y: number; height: number }) {
  const listRect = list.getBoundingClientRect();
  const itemTop = layout.y;
  const itemBottom = layout.y + layout.height;
  if (itemTop < listRect.top + 8) {
    list.scrollTop -= listRect.top - itemTop + 16;
  } else if (itemBottom > listRect.bottom - 8) {
    list.scrollTop += itemBottom - listRect.bottom + 16;
  }
  updateAllLayouts();
}

function scrollSectionIntoView(list: HTMLElement, section: HTMLElement) {
  const listRect = list.getBoundingClientRect();
  const sectionRect = section.getBoundingClientRect();
  list.scrollTop += sectionRect.top - listRect.top;
  updateAllLayouts();
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
    onArrowPress: (direction) => {
      // Keep vertical navigation in the list; use the index rail explicitly.
      if (direction === 'right') {
        return false;
      }
      return true;
    },
    onFocus: (layout) => {
      const list = listRef.current;
      if (list) {
        scrollItemIntoView(list, layout);
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

function LetterBlock({
  group,
  server,
  listRef,
  onSelect,
}: {
  group: LetterGroup;
  server: string;
  listRef: RefObject<HTMLDivElement>;
  onSelect: (movie: MovieSummary) => void;
}) {
  return (
    <div className={styles.section} data-letter={group.letter}>
      <h2 className={styles.letterHeading}>{group.letter}</h2>
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
    </div>
  );
}

function DisabledIndexLetter({ letter }: { letter: string }) {
  return (
    <span className={`${styles.indexLetter} ${styles.indexDisabled}`} aria-hidden>
      {letter}
    </span>
  );
}

function EnabledIndexLetter({
  letter,
  onJump,
}: {
  letter: string;
  onJump: (letter: string) => void;
}) {
  const { ref, focused } = useFocusable({
    focusKey: `index-${letter}`,
    onEnterPress: () => onJump(letter),
    onArrowPress: (direction) => direction !== 'left',
  });

  return (
    <button
      ref={ref}
      type="button"
      className={`${styles.indexLetter} ${styles.indexEnabled} ${focused ? styles.indexFocused : ''}`}
      onClick={() => onJump(letter)}
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
    scrollSectionIntoView(list, section);

    const first = groups.find((group) => group.letter === letter)?.movies[0];
    if (first) {
      window.requestAnimationFrame(() => {
        setFocus(itemFocusKey(first.slug));
      });
    }
  };

  if (movies.length === 0) {
    return <p className={styles.empty}>No movies found.</p>;
  }

  return (
    <FocusContext.Provider value={focusKey}>
      <div ref={ref} className={styles.shell}>
        <div ref={listRef} className={styles.list}>
          {groups.map((group) => (
            <LetterBlock
              key={group.letter}
              group={group}
              server={server}
              listRef={listRef}
              onSelect={onSelect}
            />
          ))}
        </div>
        <aside className={styles.indexRail} aria-label="Jump to letter">
          <div className={styles.indexLetters}>
            {ALPHABET_LETTERS.map((letter) =>
              activeLetters.has(letter) ? (
                <EnabledIndexLetter key={letter} letter={letter} onJump={scrollToLetter} />
              ) : (
                <DisabledIndexLetter key={letter} letter={letter} />
              ),
            )}
          </div>
        </aside>
      </div>
    </FocusContext.Provider>
  );
}
