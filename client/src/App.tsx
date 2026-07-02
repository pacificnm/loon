import { useCallback, useState } from 'react';
import { HashRouter, Navigate, Route, Routes, useLocation, useParams } from 'react-router-dom';
import { ErrorBoundary } from './components/ErrorBoundary';
import { AppShell } from './components/layout/AppShell';
import { useWebOsLifecycle } from './platform/useWebOsLifecycle';
import { useWebOsBack } from './platform/useWebOsBack';
import { AdminPage } from './pages/AdminPage';
import { FavoritesPage } from './pages/FavoritesPage';
import { GenresPage } from './pages/GenresPage';
import { MovieDetailPage } from './pages/MovieDetailPage';
import { MovieEditPage } from './pages/MovieEditPage';
import { MoviesPage } from './pages/MoviesPage';
import { PersonPage } from './pages/PersonPage';
import { PlayerPage } from './pages/PlayerPage';
import { SearchPage } from './pages/SearchPage';
import type { MovieDetail } from './api/types';

function MovieDetailRoute() {
  const { slug = '' } = useParams();
  const location = useLocation();
  const state = location.state as {
    refreshedMovie?: MovieDetail;
    refreshEpoch?: number;
  } | null;
  const refreshEpoch = state?.refreshEpoch ?? 0;
  const refreshedMovie =
    state?.refreshedMovie?.slug === slug ? state.refreshedMovie : undefined;

  return (
    <MovieDetailPage
      key={`${slug}-${refreshEpoch}`}
      refreshedMovie={refreshedMovie}
      refreshEpoch={refreshEpoch}
    />
  );
}

function GenreMoviesRoute({ focusEpoch }: { focusEpoch: number }) {
  const { name = '' } = useParams();
  return <MoviesPage focusEpoch={focusEpoch} genre={decodeURIComponent(name)} />;
}

function AppRoutes() {
  const [focusEpoch, setFocusEpoch] = useState(0);

  const handleRelaunch = useCallback(() => {
    setFocusEpoch((epoch) => epoch + 1);
  }, []);

  useWebOsLifecycle(handleRelaunch);
  useWebOsBack();

  return (
    <Routes>
      <Route element={<AppShell focusEpoch={focusEpoch} />}>
        <Route index element={<MoviesPage focusEpoch={focusEpoch} />} />
        <Route path="search" element={<SearchPage focusEpoch={focusEpoch} />} />
        <Route path="genres" element={<GenresPage focusEpoch={focusEpoch} />} />
        <Route path="genre/:name" element={<GenreMoviesRoute focusEpoch={focusEpoch} />} />
        <Route path="favorites" element={<FavoritesPage focusEpoch={focusEpoch} />} />
        <Route path="admin" element={<AdminPage />} />
        <Route path="movie/:slug/edit" element={<MovieEditPage />} />
        <Route path="movie/:slug" element={<MovieDetailRoute />} />
        <Route path="person/:tmdbId" element={<PersonPage />} />
      </Route>
      <Route path="play/:slug" element={<PlayerPage />} />
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

export function App() {
  return (
    <HashRouter>
      <AppWithErrorBoundary />
    </HashRouter>
  );
}

function AppWithErrorBoundary() {
  const location = useLocation();

  return (
    <ErrorBoundary resetKeys={[location.pathname, location.search, location.hash]}>
      <AppRoutes />
    </ErrorBoundary>
  );
}
