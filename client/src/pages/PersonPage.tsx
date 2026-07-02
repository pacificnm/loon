import { useCallback, useEffect, useState } from 'react';
import { useLocation, useNavigate, useParams } from 'react-router-dom';
import {
  FocusContext,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import { fetchPerson, fetchPersonForCast } from '../api/client';
import type { KnownForMovie, PersonDetail } from '../api/types';
import { FocusButton } from '../components/FocusButton';
import { getServerUrl, resolveArtworkUrl } from '../config';
import styles from './PersonPage.module.css';

function parsePersonRouteId(raw: string): number | null {
  const trimmed = raw.trim();
  if (!trimmed || trimmed === 'lookup') {
    return null;
  }
  const numeric = trimmed.startsWith('tmdb:') ? trimmed.slice('tmdb:'.length) : trimmed;
  const personId = Number.parseInt(numeric, 10);
  return Number.isFinite(personId) && personId > 0 ? personId : null;
}

function formatGender(code?: number): string | null {
  switch (code) {
    case 1:
      return 'Male';
    case 2:
      return 'Female';
    case 3:
      return 'Non-binary';
    default:
      return null;
  }
}

export function PersonPage() {
  const { tmdbId = '' } = useParams();
  const location = useLocation();
  const castLookup = location.state as { movieSlug?: string; castName?: string } | null;
  const isLookup = tmdbId === 'lookup';
  const personId = parsePersonRouteId(tmdbId);
  const server = getServerUrl();
  const navigate = useNavigate();
  const [detail, setDetail] = useState<PersonDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const { ref, focusKey, focusSelf } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'person-page',
    preferredChildFocusKey: 'person-known-0',
  });

  const resolveArtwork = useCallback(
    (path: string | undefined) => resolveArtworkUrl(path, server, detail?.tmdb_person_id),
    [detail?.tmdb_person_id, server],
  );

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    const load = isLookup
      ? (() => {
          const movieSlug = castLookup?.movieSlug;
          const castName = castLookup?.castName;
          if (!movieSlug || !castName) {
            return Promise.reject(new Error('Missing cast lookup context'));
          }
          return fetchPersonForCast(server, movieSlug, castName);
        })()
      : (() => {
          if (personId == null) {
            return Promise.reject(new Error('Invalid person id'));
          }
          return fetchPerson(server, personId);
        })();

    void load
      .then((person) => {
        if (!cancelled) {
          setDetail(person);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : 'Failed to load person');
          setDetail(null);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [castLookup?.castName, castLookup?.movieSlug, isLookup, personId, server]);

  useEffect(() => {
    if (detail) {
      window.requestAnimationFrame(() => focusSelf());
    }
  }, [detail, focusSelf]);

  if (loading) {
    return <p className={styles.status}>Loading actor…</p>;
  }

  if (error || !detail) {
    return (
      <div className={styles.error}>
        <p>{error ?? 'Person not found'}</p>
        <FocusButton label="Back" onPress={() => navigate(-1)} />
      </div>
    );
  }

  const profileUrl = resolveArtwork(detail.profile_url);
  const gender = formatGender(detail.gender);

  return (
    <div className={styles.page}>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.scroll}>
          <section className={styles.layout}>
            <aside className={styles.sidebar}>
              <div className={styles.profileFrame}>
                {profileUrl ? (
                  <img className={styles.profile} src={profileUrl} alt="" />
                ) : (
                  <div className={styles.profilePlaceholder}>{detail.name.slice(0, 1)}</div>
                )}
              </div>
              <h2 className={styles.sidebarTitle}>Personal Info</h2>
              <dl className={styles.facts}>
                {detail.known_for_department ? (
                  <div className={styles.fact}>
                    <dt>Known For</dt>
                    <dd>{detail.known_for_department}</dd>
                  </div>
                ) : null}
                {gender ? (
                  <div className={styles.fact}>
                    <dt>Gender</dt>
                    <dd>{gender}</dd>
                  </div>
                ) : null}
                {detail.birthday ? (
                  <div className={styles.fact}>
                    <dt>Birthday</dt>
                    <dd>{detail.birthday}</dd>
                  </div>
                ) : null}
                {detail.place_of_birth ? (
                  <div className={styles.fact}>
                    <dt>Place of Birth</dt>
                    <dd>{detail.place_of_birth}</dd>
                  </div>
                ) : null}
                {detail.also_known_as.length > 0 ? (
                  <div className={styles.fact}>
                    <dt>Also Known As</dt>
                    <dd>{detail.also_known_as.join(', ')}</dd>
                  </div>
                ) : null}
              </dl>
            </aside>

            <div className={styles.main}>
              <h1 className={styles.name}>{detail.name}</h1>
              {detail.biography ? (
                <section className={styles.biographySection}>
                  <h2 className={styles.sectionTitle}>Biography</h2>
                  <p className={styles.biography}>{detail.biography}</p>
                </section>
              ) : null}

              {detail.known_for.length > 0 ? (
                <KnownForRow
                  movies={detail.known_for}
                  resolveArtwork={resolveArtwork}
                  onSelect={(movie) => navigate(`/movie/${movie.slug}`)}
                />
              ) : (
                <p className={styles.emptyKnownFor}>
                  No movies in your library feature this actor yet.
                </p>
              )}
            </div>
          </section>
        </div>
      </FocusContext.Provider>
    </div>
  );
}

function KnownForRow({
  movies,
  resolveArtwork,
  onSelect,
}: {
  movies: KnownForMovie[];
  resolveArtwork: (path: string | undefined) => string | undefined;
  onSelect: (movie: KnownForMovie) => void;
}) {
  const { ref, focusKey } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'person-known-row',
    preferredChildFocusKey: movies[0] ? 'person-known-0' : undefined,
  });

  return (
    <section className={styles.knownForSection}>
      <h2 className={styles.sectionTitle}>Known For</h2>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.knownForRow}>
          {movies.map((movie, index) => (
            <KnownForCard
              key={movie.slug}
              movie={movie}
              index={index}
              resolveArtwork={resolveArtwork}
              onSelect={onSelect}
            />
          ))}
        </div>
      </FocusContext.Provider>
    </section>
  );
}

function KnownForCard({
  movie,
  index,
  resolveArtwork,
  onSelect,
}: {
  movie: KnownForMovie;
  index: number;
  resolveArtwork: (path: string | undefined) => string | undefined;
  onSelect: (movie: KnownForMovie) => void;
}) {
  const posterUrl = resolveArtwork(movie.poster_url);
  const { ref, focused } = useFocusable({
    focusKey: `person-known-${index}`,
    onEnterPress: () => onSelect(movie),
  });

  return (
    <div
      ref={ref}
      className={`${styles.knownForCard} ${focused ? styles.knownForCardFocused : ''}`}
      role="button"
      tabIndex={-1}
      onClick={() => onSelect(movie)}
    >
      <div className={styles.knownForPosterFrame}>
        {posterUrl ? (
          <img className={styles.knownForPoster} src={posterUrl} alt="" />
        ) : (
          <div className={styles.knownForPosterPlaceholder}>{movie.title.slice(0, 1)}</div>
        )}
      </div>
      <div className={styles.knownForText}>
        <span className={styles.knownForTitle}>{movie.title}</span>
        {movie.character ? (
          <span className={styles.knownForRole}>{movie.character}</span>
        ) : null}
      </div>
    </div>
  );
}
