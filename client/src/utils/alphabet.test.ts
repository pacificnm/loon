import { describe, expect, it } from 'vitest';
import { groupMoviesByLetter, letterForTitle } from './alphabet';

describe('alphabet utils', () => {
  it('maps titles to A-Z or hash bucket', () => {
    expect(letterForTitle('Alien')).toBe('A');
    expect(letterForTitle('the matrix')).toBe('T');
    expect(letterForTitle('9 Lives')).toBe('#');
  });

  it('groups sorted movies by letter', () => {
    const groups = groupMoviesByLetter([
      { slug: 'a', title: 'Alien', runtime_minutes: 1, summary: '' },
      { slug: 'a2', title: 'Arrival', runtime_minutes: 1, summary: '' },
      { slug: 'b', title: 'Blade Runner', runtime_minutes: 1, summary: '' },
      { slug: 'm', title: 'Matrix', runtime_minutes: 1, summary: '' },
    ]);

    expect(groups).toEqual([
      {
        letter: 'A',
        movies: [
          { slug: 'a', title: 'Alien', runtime_minutes: 1, summary: '' },
          { slug: 'a2', title: 'Arrival', runtime_minutes: 1, summary: '' },
        ],
      },
      {
        letter: 'B',
        movies: [{ slug: 'b', title: 'Blade Runner', runtime_minutes: 1, summary: '' }],
      },
      {
        letter: 'M',
        movies: [{ slug: 'm', title: 'Matrix', runtime_minutes: 1, summary: '' }],
      },
    ]);
  });
});
