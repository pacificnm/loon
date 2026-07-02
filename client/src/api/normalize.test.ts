import { describe, expect, it } from 'vitest';
import { normalizeMovieDetail } from './normalize';
import type { MovieDetail } from './types';

const baseDetail: MovieDetail = {
  slug: 'alien-1979',
  title: 'Alien',
  genres: ['Horror'],
  cast: [],
  crew: [],
  is_favorite: false,
  stream_url: '/stream/alien-1979',
};

describe('normalizeMovieDetail', () => {
  it('fills missing file info for older servers', () => {
    const detail = normalizeMovieDetail(baseDetail);
    expect(detail.file?.filename).toBe('alien-1979');
    expect(detail.file?.content_type).toBe('application/octet-stream');
  });

  it('preserves server file info when present', () => {
    const detail = normalizeMovieDetail({
      ...baseDetail,
      file: {
        filename: 'Alien (1979).mp4',
        relative_path: 'Movies/Alien (1979)/Alien (1979).mp4',
        extension: 'mp4',
        size_bytes: 1024,
        content_type: 'video/mp4',
        modified_at: 1,
        scanned_at: 2,
      },
    });
    expect(detail.file?.filename).toBe('Alien (1979).mp4');
    expect(detail.file?.size_bytes).toBe(1024);
  });
});
