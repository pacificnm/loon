import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import {
  faFilm,
  faArrowRotateRight,
  faGear,
} from '@fortawesome/free-solid-svg-icons'
import { WindowControls } from './WindowControls'
import { isTauri } from '../lib/tauri'

type Section = 'library' | 'scan' | 'settings'

interface AppShellProps {
  activeSection: Section
  onSectionChange: (section: Section) => void
  title: string
  subtitle: string
  children: React.ReactNode
}

const NAV_ITEMS: { id: Section; label: string; icon: typeof faFilm }[] = [
  { id: 'library', label: 'Library', icon: faFilm },
  { id: 'scan', label: 'Scan', icon: faArrowRotateRight },
  { id: 'settings', label: 'Settings', icon: faGear },
]

const TITLE_BAR_HEIGHT = 32

export function AppShell({
  activeSection,
  onSectionChange,
  title,
  subtitle,
  children,
}: AppShellProps) {
  const showWindowChrome = isTauri()

  return (
    <div className="flex h-full flex-col bg-loon-bg">
      <header
        className="relative flex shrink-0 items-stretch border-b border-loon-border bg-loon-surface text-[13px]"
        style={{ height: TITLE_BAR_HEIGHT }}
      >
        {showWindowChrome ? (
          <div className="min-w-0 flex-1" data-tauri-drag-region />
        ) : (
          <div className="min-w-0 flex-1 px-4 flex items-center">
            <div>
              <h1 className="text-sm font-semibold text-loon-fg leading-tight">{title}</h1>
              <p className="text-[11px] text-loon-muted leading-tight">{subtitle}</p>
            </div>
          </div>
        )}

        <div className="relative z-10 flex h-full shrink-0 items-stretch">
          {showWindowChrome ? <WindowControls /> : null}
        </div>

        {showWindowChrome ? (
          <p
            className="pointer-events-none absolute inset-0 z-0 flex flex-col items-center justify-center px-32"
            aria-hidden
          >
            <span className="truncate text-[12px] font-medium text-loon-fg">
              {title}
            </span>
            <span className="truncate text-[10px] text-loon-muted">
              {subtitle}
            </span>
          </p>
        ) : null}
      </header>

      <div className="flex flex-1 overflow-hidden">
        <nav className="flex w-60 flex-col border-r border-loon-border bg-loon-surface">
          <div className="flex-1 py-3">
            {NAV_ITEMS.map((item) => (
              <button
                key={item.id}
                onClick={() => onSectionChange(item.id)}
                className={`flex w-full items-center gap-3 px-4 py-2.5 text-sm transition-colors ${
                  activeSection === item.id
                    ? 'bg-loon-primary/10 text-loon-primary'
                    : 'text-loon-fg hover:bg-loon-border/50'
                }`}
              >
                <FontAwesomeIcon icon={item.icon} className="h-4 w-4" />
                <span>{item.label}</span>
              </button>
            ))}
          </div>

          <div className="border-t border-loon-border p-3">
            <p className="text-xs text-loon-muted">Loon Admin</p>
          </div>
        </nav>

        <main className="flex-1 overflow-auto p-6">{children}</main>
      </div>
    </div>
  )
}
