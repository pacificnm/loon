/** Format byte counts for display. */
export function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 ** 2) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  if (bytes < 1024 ** 3) {
    return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
  }
  return `${(bytes / 1024 ** 3).toFixed(2)} GB`;
}

/** Format unix seconds as a local date/time string. */
export function formatUnixTime(seconds: number): string {
  return new Date(seconds * 1000).toLocaleString();
}

/** Format seconds as m:ss or h:mm:ss. */
export function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
  }
  return `${minutes}:${String(secs).padStart(2, '0')}`;
}

/** Uppercase file extension labels for display. */
export function formatExtension(extension: string | undefined): string | null {
  if (!extension) {
    return null;
  }
  return extension.toUpperCase();
}
