import { useCallback, useState } from 'react';
import { HashRouter, Navigate, Route, Routes, useParams } from 'react-router-dom';
import { AppShell } from './components/layout/AppShell';
import { useWebOsLifecycle } from './platform/useWebOsLifecycle';
import { useWebOsBack } from './platform/useWebOsBack';
import { AdminPage } from './pages/AdminPage';
import { FavoritesPage } from './pages/FavoritesPage';
import { GenresPage } from './pages/GenresPage';
import { MovieDetailPage } from './pages/MovieDetailPage';
import { MoviesPage } from './pages/MoviesPage';
import { PlayerPage } from './pages/PlayerPage';
import { SearchPage } from './pages/SearchPage';

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
        <Route path="movie/:slug" element={<MovieDetailPage />} />
      </Route>
      <Route path="play/:slug" element={<PlayerPage />} />
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

export function App() {
  return (
    <HashRouter>
      <AppRoutes />
    </HashRouter>
  );
}
