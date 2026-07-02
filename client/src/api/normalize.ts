import type { MovieDetail, MovieFileInfo } from './types';

function basename(path: string): string {
  const normalized = path.replace(/\\/g, '/');
  const slash = normalized.lastIndexOf('/');
  return slash >= 0 ? normalized.slice(slash + 1) : normalized;
}

function extensionFromPath(path: string): string | undefined {
  const name = basename(path);
  const dot = name.lastIndexOf('.');
  if (dot <= 0) {
    return undefined;
  }
  return name.slice(dot + 1).toLowerCase();
}

function contentTypeForExtension(ext: string | undefined): string {
  switch (ext) {
    case 'mp4':
    case 'm4v':
    case 'mov':
      return 'video/mp4';
    case 'mkv':
      return 'video/x-matroska';
    case 'webm':
      return 'video/webm';
    case 'avi':
      return 'video/x-msvideo';
    default:
      return 'application/octet-stream';
  }
}

/** Build minimal file info when an older server omits the `file` object. */
export function fallbackFileInfo(slug: string): MovieFileInfo {
  const filename = slug;
  const extension = extensionFromPath(filename);
  return {
    filename,
    relative_path: filename,
    extension: extension ?? null,
    size_bytes: null,
    content_type: contentTypeForExtension(extension),
    modified_at: null,
    scanned_at: null,
  };
}

/** Normalize API payloads so newer UI works with older servers. */
export function normalizeMovieDetail(raw: MovieDetail): MovieDetail {
  return {
    ...raw,
    cast: raw.cast ?? [],
    crew: raw.crew ?? [],
    genres: raw.genres ?? [],
    file: raw.file ?? fallbackFileInfo(raw.slug),
  };
}
