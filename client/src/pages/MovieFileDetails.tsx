import { formatBytes, formatDuration, formatExtension, formatUnixTime } from '../utils/format';
import type { MovieDetail } from '../api/types';
import styles from './MovieDetailPage.module.css';

interface DetailRow {
  label: string;
  value: string;
}

function buildDetailRows(detail: MovieDetail): DetailRow[] {
  const rows: DetailRow[] = [];
  const file = detail.file;

  if (file) {
    if (file.filename) {
      rows.push({ label: 'File name', value: file.filename });
    }
    if (file.relative_path) {
      rows.push({ label: 'Path', value: file.relative_path });
    }

    const extension = formatExtension(file.extension ?? undefined);
    if (extension) {
      rows.push({ label: 'Format', value: extension });
    }

    if (file.content_type) {
      rows.push({ label: 'Type', value: file.content_type });
    }

    if (file.size_bytes != null && file.size_bytes > 0) {
      rows.push({ label: 'Size', value: formatBytes(file.size_bytes) });
    }

    if (file.modified_at != null && file.modified_at > 0) {
      rows.push({ label: 'Modified', value: formatUnixTime(file.modified_at) });
    }

    if (file.scanned_at != null && file.scanned_at > 0) {
      rows.push({ label: 'Last scanned', value: formatUnixTime(file.scanned_at) });
    }
  }

  if (detail.tmdb_id) {
    rows.push({ label: 'TMDB', value: detail.tmdb_id });
  }

  if (detail.imdb_id) {
    rows.push({ label: 'IMDb', value: detail.imdb_id });
  }

  const directors = (detail.crew ?? [])
    .filter((member) => member.job?.toLowerCase() === 'director')
    .map((member) => member.name);
  if (directors.length > 0) {
    rows.push({ label: 'Director', value: directors.join(', ') });
  }

  if (detail.watch_progress_seconds != null && detail.watch_progress_seconds > 0) {
    rows.push({
      label: 'Watch progress',
      value: formatDuration(detail.watch_progress_seconds),
    });
  }

  return rows;
}

export function FileDetailsSection({ detail }: { detail: MovieDetail }) {
  const rows = buildDetailRows(detail);
  if (rows.length === 0) {
    return null;
  }

  return (
    <section className={styles.detailsSection}>
      <h2 className={styles.sectionTitle}>File &amp; media info</h2>
      <dl className={styles.detailsGrid}>
        {rows.map((row) => (
          <div key={row.label} className={styles.detailItem}>
            <dt className={styles.detailLabel}>{row.label}</dt>
            <dd className={styles.detailValue}>{row.value}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
