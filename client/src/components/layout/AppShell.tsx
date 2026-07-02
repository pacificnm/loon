import { Outlet, useNavigate } from 'react-router-dom';
import { TopBar } from './TopBar';
import styles from './AppShell.module.css';

interface AppShellProps {
  focusEpoch?: number;
}

export function AppShell({ focusEpoch }: AppShellProps) {
  const navigate = useNavigate();

  return (
    <div className={styles.shell}>
      <TopBar focusEpoch={focusEpoch} onNavigate={(path) => navigate(path)} />
      <main className={styles.main}>
        <Outlet />
      </main>
    </div>
  );
}
