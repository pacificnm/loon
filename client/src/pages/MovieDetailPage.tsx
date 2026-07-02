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
import { CrewCreditsSection, FileDetailsSection } from './MovieFileDetails';
import styles from './MovieDetailPage.module.css';

interface MovieDetailPageProps {
  /** Fresh detail from TMDB rematch — shown immediately while revalidating. */
  refreshedMovie?: MovieDetail;
  refreshEpoch?: number;
}

export function MovieDetailPage({
  refreshedMovie,
  refreshEpoch = 0,
}: MovieDetailPageProps) {
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
    setError(null);
    setSimilar([]);
    ref.current?.scrollTo(0, 0);

    const seed = refreshedMovie?.slug === slug ? refreshedMovie : undefined;
    if (seed) {
      setDetail(seed);
      setLoading(false);
      try {
        const related = await fetchSimilarMovies(server, seed);
        setSimilar(related);
      } catch {
        setSimilar([]);
      }
    } else {
      setDetail(null);
      setLoading(true);
    }

    try {
      const movie = await fetchMovie(server, slug, {
        cacheBust: refreshEpoch > 0 ? refreshEpoch : Date.now(),
      });
      setDetail(movie);
      const related = await fetchSimilarMovies(server, movie);
      setSimilar(related);
    } catch (err) {
      if (!seed) {
        setError(err instanceof Error ? err.message : 'Failed to load movie');
        setDetail(null);
        setSimilar([]);
      }
    } finally {
      setLoading(false);
    }
  }, [ref, refreshEpoch, refreshedMovie, server, slug]);

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

  const artworkVersion = detail?.tmdb_id ?? refreshEpoch;

  const resolveArtwork = useCallback(
    (path: string | undefined) => resolveArtworkUrl(path, server, artworkVersion),
    [artworkVersion, server],
  );

  if (loading && !detail) {
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
  const backdropUrl = resolveArtwork(detail.backdrop_url);
  const runtime =
    detail.runtime_minutes && detail.runtime_minutes > 0
      ? `${Math.floor(detail.runtime_minutes / 60)}h ${detail.runtime_minutes % 60}m`
      : null;
  const hasOverview = Boolean(detail.summary) || (detail.crew?.length ?? 0) > 0;

  return (
    <div className={styles.page}>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.scroll}>
          <section className={styles.backdropHero}>
            {backdropUrl ? (
              <img key={backdropUrl} className={styles.backdrop} src={backdropUrl} alt="" />
            ) : (
              <div className={styles.backdropFallback} />
            )}
            <div className={styles.backdropScrim} />
            <div className={styles.heroContent}>
              <div className={styles.posterFrame}>
                {posterUrl ? (
                  <img
                    key={posterUrl}
                    className={styles.poster}
                    src={posterUrl}
                    alt=""
                  />
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
                  {[detail.year, runtime, (detail.genres ?? []).join(' · ')].filter(Boolean).join(' · ')}
                </p>
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
                  <FocusButton
                    focusKey="detail-edit"
                    label="Edit"
                    onPress={() => navigate(`/movie/${detail.slug}/edit`)}
                  />
                </div>
              </div>
            </div>
          </section>

          {hasOverview ? (
            <section className={styles.overviewSection}>
              <h2 className={styles.sectionTitle}>Overview</h2>
              {detail.summary ? <p className={styles.summary}>{detail.summary}</p> : null}
              <CrewCreditsSection crew={detail.crew ?? []} />
            </section>
          ) : null}

          <FileDetailsSection detail={detail} />

          {detail.cast && detail.cast.length > 0 ? (
            <CastRow cast={detail.cast} resolveArtwork={resolveArtwork} />
          ) : null}

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

function CastRow({
  cast,
  resolveArtwork,
}: {
  cast: MovieDetail['cast'];
  resolveArtwork: (path: string | undefined) => string | undefined;
}) {
  const { ref, focusKey } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'cast-row',
    preferredChildFocusKey: cast[0] ? `cast-0` : undefined,
  });

  return (
    <section className={styles.castSection}>
      <h2 className={styles.sectionTitle}>Top Billed Cast</h2>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.castRow}>
          {cast.map((member, index) => (
            <CastCard
              key={`${member.name}-${index}`}
              member={member}
              index={index}
              resolveArtwork={resolveArtwork}
            />
          ))}
        </div>
      </FocusContext.Provider>
    </section>
  );
}

function CastCard({
  member,
  index,
  resolveArtwork,
}: {
  member: MovieDetail['cast'][number];
  index: number;
  resolveArtwork: (path: string | undefined) => string | undefined;
}) {
  const { ref, focused } = useFocusable({
    focusKey: `cast-${index}`,
  });
  const profileUrl = resolveArtwork(member.profile_url);

  return (
    <div ref={ref} className={`${styles.castCard} ${focused ? styles.castCardFocused : ''}`}>
      <div className={styles.castPhotoFrame}>
        {profileUrl ? (
          <img className={styles.castPhoto} src={profileUrl} alt="" />
        ) : (
          <div className={styles.castPhotoPlaceholder}>{member.name.slice(0, 1)}</div>
        )}
      </div>
      <div className={styles.castText}>
        <span className={styles.castName}>{member.name}</span>
        {member.character ? (
          <span className={styles.castRole}>{member.character}</span>
        ) : null}
      </div>
    </div>
  );
}
