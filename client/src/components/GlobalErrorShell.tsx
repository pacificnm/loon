import { useEffect, useState, type ReactNode } from 'react';
import { ErrorFallback } from './ErrorFallback';

function errorFromUnknown(value: unknown): Error {
  if (value instanceof Error) {
    return value;
  }
  if (typeof value === 'string') {
    return new Error(value);
  }
  try {
    return new Error(JSON.stringify(value));
  } catch {
    return new Error('Unknown error');
  }
}

interface GlobalErrorShellProps {
  children: ReactNode;
}

/** Catches window-level errors that React error boundaries never see. */
export function GlobalErrorShell({ children }: GlobalErrorShellProps) {
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const onWindowError = (event: ErrorEvent) => {
      console.error('[Loon] window error', event.error ?? event.message);
      setError(errorFromUnknown(event.error ?? event.message));
      event.preventDefault();
    };

    const onUnhandledRejection = (event: PromiseRejectionEvent) => {
      console.error('[Loon] unhandled rejection', event.reason);
      setError(errorFromUnknown(event.reason));
      event.preventDefault();
    };

    window.addEventListener('error', onWindowError);
    window.addEventListener('unhandledrejection', onUnhandledRejection);
    return () => {
      window.removeEventListener('error', onWindowError);
      window.removeEventListener('unhandledrejection', onUnhandledRejection);
    };
  }, []);

  if (error) {
    return <ErrorFallback error={error} onRetry={() => setError(null)} />;
  }

  return children;
}
