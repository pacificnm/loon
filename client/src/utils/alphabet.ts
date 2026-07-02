import type { MovieSummary } from '../api/types';

export interface LetterGroup {
  letter: string;
  movies: MovieSummary[];
}

/** First character bucket for alphabet navigation (A–Z, else `#`). */
export function letterForTitle(title: string): string {
  const first = title.trim().charAt(0).toUpperCase();
  return first >= 'A' && first <= 'Z' ? first : '#';
}

/** Group movies by first letter. Input must already be sorted by title. */
export function groupMoviesByLetter(movies: MovieSummary[]): LetterGroup[] {
  const groups: LetterGroup[] = [];

  for (const movie of movies) {
    const letter = letterForTitle(movie.title);
    const last = groups[groups.length - 1];
    if (last?.letter === letter) {
      last.movies.push(movie);
    } else {
      groups.push({ letter, movies: [movie] });
    }
  }

  return groups;
}

export const ALPHABET_LETTERS = [
  ...'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split(''),
  '#',
] as const;
