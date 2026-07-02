import { useCallback, useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  FocusContext,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import {
  fetchMovie,
  fetchSimilarMovies,
  setFavorite,
} from '../api/client';
import type { MovieDetail, MovieSummary } from '../api/types';
import { FocusButton } from '../components/FocusButton';
import { HorizontalRow } from '../components/HorizontalRow';
import { getServerUrl, resolveArtworkUrl } from '../config';
import styles from './MovieDetailPage.module.css';

export function MovieDetailPage() {
  const { slug = '' } = useParams();
  const server = getServerUrl();
  const navigate = useNavigate();
  const [detail, setDetail] = useState<MovieDetail | null>(null);
  const [similar, setSimilar] = useState<MovieSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [favoriteBusy, setFavoriteBusy] = useState(false);

  const { ref, focusKey, focusSelf } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'movie-detail',
    preferredChildFocusKey: 'detail-play',
  });

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    setDetail(null);
    setSimilar([]);
    ref.current?.scrollTo(0, 0);

    try {
      const movie = await fetchMovie(server, slug);
      setDetail(movie);
      const related = await fetchSimilarMovies(server, movie);
      setSimilar(related);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load movie');
      setDetail(null);
      setSimilar([]);
    } finally {
      setLoading(false);
    }
  }, [server, slug]);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    if (detail) {
      window.requestAnimationFrame(() => focusSelf());
    }
  }, [detail, focusSelf]);

  const toggleFavorite = async () => {
    if (!detail || favoriteBusy) {
      return;
    }
    setFavoriteBusy(true);
    try {
      const response = await setFavorite(server, detail.slug);
      setDetail({ ...detail, is_favorite: response.favorite });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update favorite');
    } finally {
      setFavoriteBusy(false);
    }
  };

  const resolveArtwork = useCallback(
    (path: string | undefined) => resolveArtworkUrl(path, server),
    [server],
  );

  if (loading) {
    return <p className={styles.status}>Loading movie…</p>;
  }

  if (error || !detail) {
    return (
      <div className={styles.error}>
        <p>{error ?? 'Movie not found'}</p>
        <FocusButton label="Back" onPress={() => navigate(-1)} />
      </div>
    );
  }

  const posterUrl = resolveArtwork(detail.poster_url);
  const runtime =
    detail.runtime_minutes && detail.runtime_minutes > 0
      ? `${Math.floor(detail.runtime_minutes / 60)}h ${detail.runtime_minutes % 60}m`
      : null;

  return (
    <div className={styles.page}>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.scroll}>
          <section className={styles.hero}>
            <div className={styles.posterFrame}>
              {posterUrl ? (
                <img className={styles.poster} src={posterUrl} alt="" />
              ) : (
                <div className={styles.posterPlaceholder}>{detail.title.slice(0, 1)}</div>
              )}
            </div>
            <div className={styles.info}>
              <h1 className={styles.title}>{detail.title}</h1>
              {detail.original_title && detail.original_title !== detail.title ? (
                <p className={styles.original}>{detail.original_title}</p>
              ) : null}
              <p className={styles.meta}>
                {[detail.year, runtime, detail.genres.join(' · ')].filter(Boolean).join(' · ')}
              </p>
              {detail.summary ? <p className={styles.summary}>{detail.summary}</p> : null}
              <div className={styles.actions}>
                <FocusButton
                  focusKey="detail-play"
                  label="Play"
                  onPress={() => navigate(`/play/${detail.slug}`)}
                />
                <FocusButton
                  focusKey="detail-favorite"
                  label={detail.is_favorite ? 'Remove Favorite' : 'Favorite'}
                  onPress={() => void toggleFavorite()}
                />
              </div>
            </div>
          </section>

          {detail.cast.length > 0 ? <CastRow cast={detail.cast} /> : null}

          {similar.length > 0 ? (
            <HorizontalRow
              title="Similar movies"
              prefix="similar"
              movies={similar}
              resolveArtwork={resolveArtwork}
              onSelect={(movie) => {
                if (movie.slug !== slug) {
                  navigate(`/movie/${movie.slug}`);
                }
              }}
            />
          ) : null}
        </div>
      </FocusContext.Provider>
    </div>
  );
}

function CastRow({ cast }: { cast: MovieDetail['cast'] }) {
  const { ref, focusKey } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'cast-row',
    preferredChildFocusKey: cast[0] ? `cast-0` : undefined,
  });

  return (
    <section className={styles.castSection}>
      <h2 className={styles.sectionTitle}>Cast</h2>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.castRow}>
          {cast.map((member, index) => (
            <CastChip key={`${member.name}-${index}`} member={member} index={index} />
          ))}
        </div>
      </FocusContext.Provider>
    </section>
  );
}

function CastChip({
  member,
  index,
}: {
  member: MovieDetail['cast'][number];
  index: number;
}) {
  const { ref, focused } = useFocusable({
    focusKey: `cast-${index}`,
  });

  return (
    <div ref={ref} className={`${styles.castChip} ${focused ? styles.castChipFocused : ''}`}>
      <span className={styles.castName}>{member.name}</span>
      {member.character ? (
        <span className={styles.castRole}>{member.character}</span>
      ) : null}
    </div>
  );
}
