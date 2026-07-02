export function getServerUrl(): string {
  const fromEnv = import.meta.env.VITE_LOON_SERVER as string | undefined;
  if (fromEnv?.trim()) {
    return fromEnv.replace(/\/$/, '');
  }
  throw new Error(
    'Missing VITE_LOON_SERVER. Copy .env.example to .env.local and set your loon-server URL.',
  );
}

export function resolveArtworkUrl(
  path: string | undefined,
  server: string,
): string | undefined {
  if (!path) {
    return undefined;
  }
  if (path.startsWith('http://') || path.startsWith('https://')) {
    return path;
  }
  return `${server}${path.startsWith('/') ? path : `/${path}`}`;
}

export function streamUrl(server: string, slug: string): string {
  return `${server}/stream/${encodeURIComponent(slug)}`;
}
