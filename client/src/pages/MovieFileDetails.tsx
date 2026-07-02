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

  if (detail.watch_progress_seconds != null && detail.watch_progress_seconds > 0) {
    rows.push({
      label: 'Watch progress',
      value: formatDuration(detail.watch_progress_seconds),
    });
  }

  return rows;
}

function splitIntoColumns<T>(items: T[], columns: number): T[][] {
  if (items.length === 0) {
    return [];
  }
  const perColumn = Math.ceil(items.length / columns);
  return Array.from({ length: columns }, (_, index) =>
    items.slice(index * perColumn, (index + 1) * perColumn),
  ).filter((column) => column.length > 0);
}

export function FileDetailsSection({ detail }: { detail: MovieDetail }) {
  const rows = buildDetailRows(detail);
  if (rows.length === 0) {
    return null;
  }

  const columns = splitIntoColumns(rows, 3);

  return (
    <section className={styles.detailsSection}>
      <h2 className={styles.sectionTitle}>File &amp; media info</h2>
      <div className={styles.detailsColumns}>
        {columns.map((column, columnIndex) => (
          <dl key={columnIndex} className={styles.detailsColumn}>
            {column.map((row) => (
              <div key={row.label} className={styles.detailItem}>
                <dt className={styles.detailLabel}>{row.label}</dt>
                <dd className={styles.detailValue}>{row.value}</dd>
              </div>
            ))}
          </dl>
        ))}
      </div>
    </section>
  );
}

interface CrewCreditsProps {
  crew: MovieDetail['crew'];
}

export function CrewCreditsSection({ crew }: CrewCreditsProps) {
  const directors = crew
    .filter((member) => member.job?.toLowerCase() === 'director')
    .map((member) => member.name);
  const producers = crew
    .filter((member) => {
      const job = member.job?.toLowerCase() ?? '';
      return job === 'producer' || job === 'executive producer';
    })
    .map((member) => member.name);

  if (directors.length === 0 && producers.length === 0) {
    return null;
  }

  return (
    <div className={styles.crewCredits}>
      {directors.length > 0 ? (
        <p className={styles.crewLine}>
          <span className={styles.crewLabel}>Director</span>
          {directors.join(', ')}
        </p>
      ) : null}
      {producers.length > 0 ? (
        <p className={styles.crewLine}>
          <span className={styles.crewLabel}>Producers</span>
          {producers.join(', ')}
        </p>
      ) : null}
    </div>
  );
}
