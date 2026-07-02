import { useEffect, useState } from 'react';
import { useLocation } from 'react-router-dom';
import {
  FocusContext,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import { FocusButton } from '../FocusButton';
import styles from './TopBar.module.css';

export type NavRoute = '/' | '/search' | '/genres' | '/favorites' | '/admin';

interface NavItem {
  label: string;
  path: NavRoute;
  focusKey: string;
}

const NAV_ITEMS: NavItem[] = [
  { label: 'Search', path: '/search', focusKey: 'nav-search' },
  { label: 'Movies', path: '/', focusKey: 'nav-movies' },
  { label: 'Genres', path: '/genres', focusKey: 'nav-genres' },
  { label: 'Favorites', path: '/favorites', focusKey: 'nav-favorites' },
];

function Clock() {
  const [time, setTime] = useState(() => formatTime());

  useEffect(() => {
    const timer = window.setInterval(() => setTime(formatTime()), 30_000);
    return () => window.clearInterval(timer);
  }, []);

  return <span className={styles.clock}>{time}</span>;
}

function formatTime(): string {
  return new Intl.DateTimeFormat(undefined, {
    hour: 'numeric',
    minute: '2-digit',
  }).format(new Date());
}

interface TopBarProps {
  onNavigate: (path: NavRoute) => void;
  focusEpoch?: number;
}

export function TopBar({ onNavigate, focusEpoch = 0 }: TopBarProps) {
  const location = useLocation();
  const { ref, focusKey, focusSelf } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'top-bar',
    preferredChildFocusKey: 'nav-movies',
  });

  useEffect(() => {
    if (location.pathname === '/') {
      focusSelf();
    }
  }, [focusSelf, focusEpoch, location.pathname]);

  return (
    <header className={styles.bar}>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.left}>
          <span className={styles.logo}>Loon</span>
          <nav className={styles.nav}>
            {NAV_ITEMS.map((item) => (
              <FocusButton
                key={item.focusKey}
                focusKey={item.focusKey}
                label={item.label}
                selected={location.pathname === item.path}
                onPress={() => onNavigate(item.path)}
              />
            ))}
          </nav>
        </div>
        <div className={styles.right}>
          <Clock />
          <FocusButton
            focusKey="nav-admin"
            label="Admin"
            selected={location.pathname === '/admin'}
            onPress={() => onNavigate('/admin')}
          />
        </div>
      </FocusContext.Provider>
    </header>
  );
}
