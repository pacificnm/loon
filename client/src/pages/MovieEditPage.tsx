import { useCallback, useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  FocusContext,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import { fetchMovie, LoonApiError, setMovieTmdbMatch } from '../api/client';
import { FocusButton } from '../components/FocusButton';
import { useServerUrl } from '../config';
import styles from './MovieEditPage.module.css';
import pageStyles from './page.module.css';

export function MovieEditPage() {
  const { slug = '' } = useParams();
  const server = useServerUrl();
  const navigate = useNavigate();
  const [tmdbId, setTmdbId] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [title, setTitle] = useState('');

  const { ref, focusKey, focusSelf } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'movie-edit-page',
    preferredChildFocusKey: 'edit-tmdb-input',
  });

  useEffect(() => {
    if (!server) {
      setLoading(false);
      setError('No server configured. Open Admin → Settings.');
      return;
    }
    let cancelled = false;
    setLoading(true);
    setError(null);

    void fetchMovie(server, slug)
      .then((movie) => {
        if (cancelled) {
          return;
        }
        setTitle(movie.title);
        setTmdbId(movie.tmdb_id ?? '');
      })
      .catch((err: unknown) => {
        if (cancelled) {
          return;
        }
        setError(err instanceof Error ? err.message : 'Failed to load movie');
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [server, slug]);

  useEffect(() => {
    if (!loading) {
      focusSelf();
    }
  }, [focusSelf, loading]);

  const submit = useCallback(async () => {
    const trimmed = tmdbId.trim();
    if (!trimmed) {
      setError('Enter a TMDB movie id');
      return;
    }
    if (!server) {
      setError('No server configured. Open Admin → Settings.');
      return;
    }

    setSaving(true);
    setError(null);
    try {
      const updated = await setMovieTmdbMatch(server, slug, trimmed);
      navigate(`/movie/${slug}`, {
        replace: true,
        state: {
          refreshedMovie: updated,
          refreshEpoch: Date.now(),
        },
      });
    } catch (err) {
      if (err instanceof LoonApiError && err.code === 'tmdb_not_configured') {
        setError('TMDB is not configured on the server');
      } else {
        setError(err instanceof Error ? err.message : 'Failed to update TMDB match');
      }
    } finally {
      setSaving(false);
    }
  }, [navigate, server, slug, tmdbId]);

  if (loading) {
    return <p className={pageStyles.status}>Loading movie…</p>;
  }

  return (
    <div className={pageStyles.page}>
      <h1 className={pageStyles.heading}>Edit TMDB match</h1>
      <p className={pageStyles.status}>{title}</p>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.panel}>
          <label className={styles.label} htmlFor="edit-tmdb-input">
            TMDB movie id
          </label>
          <p className={styles.hint}>
            Enter the numeric id from themoviedb.org (for example 348 for Alien).
          </p>
          <TmdbIdInput value={tmdbId} onChange={setTmdbId} disabled={saving} />
          {error ? <p className={styles.error}>{error}</p> : null}
          <div className={styles.actions}>
            <FocusButton
              focusKey="edit-save"
              label={saving ? 'Saving…' : 'Save'}
              onPress={() => void submit()}
            />
            <FocusButton
              focusKey="edit-cancel"
              label="Cancel"
              onPress={() => navigate(-1)}
            />
          </div>
        </div>
      </FocusContext.Provider>
    </div>
  );
}

function TmdbIdInput({
  value,
  onChange,
  disabled,
}: {
  value: string;
  onChange: (value: string) => void;
  disabled: boolean;
}) {
  const { ref, focused } = useFocusable({
    focusKey: 'edit-tmdb-input',
    focusable: !disabled,
  });

  return (
    <input
      ref={ref}
      id="edit-tmdb-input"
      className={`${styles.input} ${focused ? styles.inputFocused : ''}`}
      type="text"
      inputMode="numeric"
      placeholder="348"
      value={value}
      disabled={disabled}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}
