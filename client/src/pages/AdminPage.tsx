import {
  FocusContext,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import { useEffect } from 'react';
import { FocusButton } from '../components/FocusButton';
import styles from './page.module.css';

export function AdminPage() {
  const { ref, focusKey, focusSelf } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'admin-page',
    preferredChildFocusKey: 'admin-settings',
  });

  useEffect(() => {
    focusSelf();
  }, [focusSelf]);

  return (
    <div className={styles.page}>
      <h1 className={styles.heading}>Admin</h1>
      <p className={styles.status}>Server settings and library tools (coming soon).</p>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.adminActions}>
          <FocusButton focusKey="admin-settings" label="Settings" onPress={() => {}} />
          <FocusButton focusKey="admin-library" label="Library tools" onPress={() => {}} />
          <FocusButton focusKey="admin-scan" label="Scan library" onPress={() => {}} />
        </div>
      </FocusContext.Provider>
    </div>
  );
}
