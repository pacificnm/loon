import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  FocusContext,
  useFocusable,
} from '@noriginmedia/norigin-spatial-navigation';
import { FocusButton } from './FocusButton';
import styles from './ErrorFallback.module.css';

interface ErrorFallbackProps {
  error: Error;
  onRetry: () => void;
}

function formatErrorMessage(error: Error): string {
  if (import.meta.env.DEV) {
    return error.stack ?? error.message;
  }
  return error.message || 'Something went wrong.';
}

export function ErrorFallback({ error, onRetry }: ErrorFallbackProps) {
  const navigate = useNavigate();
  const { ref, focusKey, focusSelf } = useFocusable({
    focusable: false,
    trackChildren: true,
    focusKey: 'error-fallback',
    preferredChildFocusKey: 'error-retry',
  });

  useEffect(() => {
    focusSelf();
  }, [focusSelf]);

  const goHome = () => {
    onRetry();
    navigate('/', { replace: true });
  };

  const reloadApp = () => {
    window.location.reload();
  };

  return (
    <div className={styles.shell}>
      <FocusContext.Provider value={focusKey}>
        <div ref={ref} className={styles.panel}>
          <h1 className={styles.title}>Something went wrong</h1>
          <p className={styles.message}>
            The app hit an unexpected error. You can try again or return home.
          </p>
          <pre className={styles.details}>{formatErrorMessage(error)}</pre>
          <div className={styles.actions}>
            <FocusButton focusKey="error-retry" label="Try again" onPress={onRetry} />
            <FocusButton focusKey="error-home" label="Go home" onPress={goHome} />
            <FocusButton focusKey="error-reload" label="Reload app" onPress={reloadApp} />
          </div>
        </div>
      </FocusContext.Provider>
    </div>
  );
}
