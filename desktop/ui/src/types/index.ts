export interface MovieSummary {
  slug: string
  title: string
  year: number | null
  runtime_minutes: number
  poster_url: string | null
  backdrop_url: string | null
  summary: string | null
  relative_path: string
  size_bytes: number | null
}

export interface MovieDetail extends MovieSummary {
  original_title: string | null
  genres: string[]
  cast: CastMember[]
  crew: CrewMember[]
  is_favorite: boolean
  watch_progress_seconds: number | null
  tmdb_id: string | null
  imdb_id: string | null
  file: MovieFileInfo
  stream_url: string
}

export interface CastMember {
  name: string
  character: string | null
  profile_url: string | null
  tmdb_person_id: number | null
}

export interface CrewMember {
  name: string
  job: string | null
}

export interface MovieFileInfo {
  filename: string
  relative_path: string
  extension: string | null
  size_bytes: number | null
  content_type: string | null
  modified_at: number | null
  scanned_at: number | null
}

export interface HealthResponse {
  status: string
  service?: string
  version?: string
  movies_count: number
  library_scanned_at?: number
}

export type ScanPhase = 'discovering' | 'enriching' | 'persisting'

export interface ScanProgress {
  phase: ScanPhase | null
  files_seen: number
  candidates: number
  errors: number
  enriched: number
  total_to_enrich: number
  current_path: string | null
}

export interface ScanStreamEvent {
  type: 'started' | 'progress' | 'complete' | 'error'
  scan_id?: string
  progress?: ScanProgress
  movies_count?: number
  duration_secs?: number
  stats?: ScanStats
  message?: string
}

export interface ScanStats {
  files_seen: number
  candidates: number
  errors: number
}

export interface LibraryStatusResponse {
  state: 'idle' | 'scanning'
  last_scan_at: string | null
  last_scan_duration_secs: number
  movies_count: number
  scan_in_progress: boolean
  progress: ScanProgress | null
}
