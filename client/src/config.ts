import { useSyncExternalStore } from 'react';

export const LOON_SERVER_URL_KEY = 'loon_server_url';

const listeners = new Set<() => void>();

function notifyServerUrlChange(): void {
  listeners.forEach((listener) => listener());
}

/** Subscribe to server URL changes (localStorage updates). */
export function subscribeServerUrl(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** Normalizes and validates a loon-server base URL. */
export function normalizeServerUrl(raw: string): string | null {
  const trimmed = raw.trim().replace(/\/$/, '');
  if (!trimmed) {
    return null;
  }
  if (!trimmed.startsWith('http://') && !trimmed.startsWith('https://')) {
    return null;
  }
  return trimmed;
}

function readStoredServerUrl(): string | null {
  if (typeof localStorage === 'undefined') {
    return null;
  }
  const stored = localStorage.getItem(LOON_SERVER_URL_KEY);
  if (!stored?.trim()) {
    return null;
  }
  return normalizeServerUrl(stored);
}

function readEnvServerUrl(): string | null {
  const fromEnv = import.meta.env.VITE_LOON_SERVER as string | undefined;
  if (!fromEnv?.trim()) {
    return null;
  }
  return normalizeServerUrl(fromEnv);
}

/** Returns the configured server URL, or null when unset. */
export function getServerUrlOrNull(): string | null {
  return readStoredServerUrl() ?? readEnvServerUrl();
}

/** Returns the configured server URL or throws. */
export function getServerUrl(): string {
  const url = getServerUrlOrNull();
  if (!url) {
    throw new Error(
      'No server URL configured. Open Admin → Settings and enter your loon-server URL.',
    );
  }
  return url;
}

/** Persists the server URL and notifies subscribers. */
export function setServerUrl(raw: string): string {
  if (typeof localStorage === 'undefined') {
    throw new Error('localStorage is not available');
  }
  const url = normalizeServerUrl(raw);
  if (!url) {
    throw new Error('Server URL must start with http:// or https://');
  }
  localStorage.setItem(LOON_SERVER_URL_KEY, url);
  notifyServerUrlChange();
  return url;
}

/** Clears a saved server URL override (env fallback may still apply). */
export function clearServerUrl(): void {
  if (typeof localStorage === 'undefined') {
    return;
  }
  localStorage.removeItem(LOON_SERVER_URL_KEY);
  notifyServerUrlChange();
}

/** Reactive server URL for React components. */
export function useServerUrl(): string | null {
  return useSyncExternalStore(
    subscribeServerUrl,
    getServerUrlOrNull,
    getServerUrlOrNull,
  );
}

export function resolveArtworkUrl(
  path: string | undefined,
  server: string,
  cacheVersion?: string | number,
): string | undefined {
  if (!path) {
    return undefined;
  }
  let url: string;
  if (path.startsWith('http://') || path.startsWith('https://')) {
    url = path;
  } else {
    url = `${server}${path.startsWith('/') ? path : `/${path}`}`;
  }
  if (cacheVersion === undefined) {
    return url;
  }
  const separator = url.includes('?') ? '&' : '?';
  return `${url}${separator}v=${encodeURIComponent(String(cacheVersion))}`;
}

export function streamUrl(server: string, slug: string): string {
  return `${server}/stream/${encodeURIComponent(slug)}`;
}
