# Platform Integration (webOS)

## Platform Module

**File:** `src/platform/webos.ts`

## webOS Platform APIs

### Global Type Declarations

```typescript
declare global {
  interface Window {
    PalmSystem?: {
      activate: () => void;
      platformBack?: () => void;
    };
    webOSSystem?: {
      activate: () => void;
      platformBack?: () => void;
    };
  }
}
```

### activateApp

**Purpose:** Signals webOS to display the app after launch.

```typescript
function activateApp(): void {
  window.webOSSystem?.activate?.();
  window.PalmSystem?.activate?.();
}
```

**Required when:** `appinfo.json` has `handlesRelaunch: true`

---

### exitWebOsApp

**Purpose:** Exits the app (returns to webOS home).

```typescript
export function exitWebOsApp(): void {
  window.webOSSystem?.platformBack?.();
  window.PalmSystem?.platformBack?.();
}
```

---

### registerWebOsLifecycle

**Signature:**
```typescript
export function registerWebOsLifecycle(
  onRelaunch: WebOsRelaunchHandler,
): () => void
```

**Purpose:** Registers handlers for webOS lifecycle events.

**Events:**
- `webOSLaunch` - Initial app launch
- `webOSRelaunch` - App relaunched from background

**Implementation:**
```typescript
export function registerWebOsLifecycle(onRelaunch: WebOsRelaunchHandler): () => void {
  const handleShow = () => {
    // Required when handlesRelaunch: true
    activateApp();
    onRelaunch();
  };

  document.addEventListener('webOSRelaunch', handleShow);
  document.addEventListener('webOSLaunch', handleShow);

  // Cold start activation
  activateApp();

  return () => {
    document.removeEventListener('webOSRelaunch', handleShow);
    document.removeEventListener('webOSLaunch', handleShow);
  };
}
```

**Usage:**
```typescript
// In App.tsx
const [focusEpoch, setFocusEpoch] = useState(0);

const handleRelaunch = useCallback(() => {
  setFocusEpoch((epoch) => epoch + 1);
}, []);

useWebOsLifecycle(handleRelaunch);
```

---

### registerVisibilityHandler

**Signature:**
```typescript
export function registerVisibilityHandler(
  onHidden: () => void,
  onVisible?: () => void,
): () => void
```

**Purpose:** Handles app visibility changes (backgrounding/foregrounding).

**Implementation:**
```typescript
export function registerVisibilityHandler(
  onHidden: () => void,
  onVisible?: () => void,
): () => void {
  const onChange = () => {
    if (document.hidden) {
      onHidden();
    } else {
      onVisible?.();
    }
  };

  document.addEventListener('visibilitychange', onChange);
  return () => document.removeEventListener('visibilitychange', onChange);
}
```

**Usage (VideoPlayer):**
```typescript
useWebOsVisibility(() => {
  videoRef.current?.pause();
});
```

---

## Lifecycle Hooks

**File:** `src/platform/useWebOsLifecycle.ts`

### useWebOsLifecycle

**Signature:**
```typescript
export function useWebOsLifecycle(onRelaunch: WebOsRelaunchHandler): void
```

**Purpose:** React hook for webOS lifecycle events.

**Usage:**
```typescript
useEffect(() => registerWebOsLifecycle(onRelaunch), [onRelaunch]);
```

---

### useWebOsVisibility

**Signature:**
```typescript
export function useWebOsVisibility(
  onHidden: () => void,
  onVisible?: () => void,
): void
```

**Purpose:** React hook for visibility changes.

**Usage:**
```typescript
useWebOsVisibility(
  () => videoRef.current?.pause(),  // App hidden
  () => videoRef.current?.play(),   // App visible (optional)
);
```

---

## Back Button Handling

**File:** `src/platform/useWebOsBack.ts`

### useWebOsBack

**Signature:**
```typescript
export function useWebOsBack(): void
```

**Purpose:** Handles Magic Remote back button navigation.

**Implementation:**
```typescript
export function useWebOsBack(): void {
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      // Don't intercept back when typing
      if (shouldDeferToTextInput(event)) return;

      if (!isAppBackKey(event)) return;

      // Player has its own back handler
      if (location.pathname.startsWith('/play/')) return;

      event.preventDefault();
      event.stopPropagation();

      // At home screen: exit to webOS
      const atHome = location.pathname === '/' || location.pathname === '';
      if (atHome) {
        window.PalmSystem?.platformBack?.();
        return;
      }

      // Otherwise: navigate back in app
      navigate(-1);
    };

    window.addEventListener('keydown', onKeyDown, true);
    return () => window.removeEventListener('keydown', onKeyDown, true);
  }, [location.pathname, navigate]);
}
```

**Usage:**
```typescript
// In App.tsx
useWebOsBack();
```

---

## Keyboard Utilities

**File:** `src/platform/keyboard.ts`

### Constants

```typescript
export const WEBOS_BACK_KEYCODE = 461;
```

---

### isEditableElement

**Signature:**
```typescript
export function isEditableElement(target: EventTarget | null): boolean
```

**Purpose:** Checks if target is a text input (to skip global shortcuts).

**Implementation:**
```typescript
export function isEditableElement(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;

  const element = target.closest(
    'input, textarea, select, [contenteditable=""], [contenteditable="true"]'
  ) ?? target;

  if (!(element instanceof HTMLElement)) return false;
  if (element.isContentEditable) return true;

  if (element instanceof HTMLInputElement) {
    const type = element.type.toLowerCase();
    return type !== 'button' && type !== 'submit' &&
           type !== 'checkbox' && type !== 'radio';
  }

  return element instanceof HTMLTextAreaElement ||
         element instanceof HTMLSelectElement;
}
```

---

### isWebOsBackKey

**Signature:**
```typescript
export function isWebOsBackKey(event: KeyboardEvent): boolean
```

**Purpose:** Detects Magic Remote back button.

**Implementation:**
```typescript
export function isWebOsBackKey(event: KeyboardEvent): boolean {
  return (
    event.keyCode === WEBOS_BACK_KEYCODE ||
    event.key === 'GoBack' ||
    event.key === 'BrowserBack'
  );
}
```

**Key Codes:**
| Key | keyCode | key |
|-----|---------|-----|
| Magic Remote Back | 461 | - |
| Browser Back | - | 'BrowserBack' |
| Go Back | - | 'GoBack' |

---

### isDevBackKey

**Signature:**
```typescript
export function isDevBackKey(event: KeyboardEvent): boolean
```

**Purpose:** Escape key for desktop development testing.

**Implementation:**
```typescript
export function isDevBackKey(event: KeyboardEvent): boolean {
  return import.meta.env.DEV && event.key === 'Escape';
}
```

---

### isAppBackKey

**Signature:**
```typescript
export function isAppBackKey(event: KeyboardEvent): boolean
```

**Purpose:** Unified back key detection.

**Implementation:**
```typescript
export function isAppBackKey(event: KeyboardEvent): boolean {
  return isWebOsBackKey(event) || isDevBackKey(event);
}
```

---

### shouldDeferToTextInput

**Signature:**
```typescript
export function shouldDeferToTextInput(event: KeyboardEvent): boolean
```

**Purpose:** Skip global shortcuts when user is typing.

**Implementation:**
```typescript
export function shouldDeferToTextInput(event: KeyboardEvent): boolean {
  return isEditableElement(event.target) ||
         isEditableElement(document.activeElement);
}
```

---

## App Configuration

**File:** `appinfo.json`

```json
{
  "id": "com.pacificnm.loon",
  "version": "0.3.5",
  "vendor": "Pacific NM",
  "type": "web",
  "main": "index.html",
  "title": "Loon",
  "icon": "icon.png",
  "largeIcon": "icon-large.png",
  "appDescription": "Personal movie streaming for LG webOS",
  "bgColor": "#090B10",
  "iconColor": "#7DD3FC",
  "resolution": "1920x1080",
  "disableBackHistoryAPI": true,
  "handlesRelaunch": true,
  "requiredPermissions": ["network.operation", "network.query"]
}
```

### Key Settings

| Field | Value | Purpose |
|-------|-------|---------|
| `type` | `"web"` | Web app (not native) |
| `main` | `"index.html"` | Entry point |
| `resolution` | `"1920x1080"` | Target resolution |
| `disableBackHistoryAPI` | `true` | Custom back button handling |
| `handlesRelaunch` | `true` | App manages relaunch events |

### Permissions

| Permission | Purpose |
|------------|---------|
| `network.operation` | HTTP requests to server |
| `network.query` | Network status checks |

---

## Build Scripts

**Directory:** `scripts/`

### prepare-webos-package.mjs

**Purpose:** Prepares the `package/` directory for deployment.

**Steps:**
1. Copy built files from `dist/` to `package/`
2. Copy `appinfo.json`
3. Copy icons (`icon.png`, `icon-large.png`)

---

### start-simulator.mjs

**Purpose:** Launches webOS Simulator.

---

### launch-simulator.mjs

**Purpose:** Packages and launches app in simulator.

**Steps:**
1. Fix simulator path
2. Build package
3. Launch in simulator

---

### fix-simulator-path.mjs

**Purpose:** Fixes webOS simulator path issues on different platforms.

---

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `VITE_LOON_SERVER` | Default server URL (build-time) |

**Usage:**
```typescript
// config.ts
function readEnvServerUrl(): string | null {
  const fromEnv = import.meta.env.VITE_LOON_SERVER;
  if (!fromEnv?.trim()) return null;
  return normalizeServerUrl(fromEnv);
}
```

---

## Server URL Configuration

**File:** `src/config.ts`

### Storage

Server URL is stored in `localStorage`:

```typescript
export const LOON_SERVER_URL_KEY = 'loon_server_url';
```

### Functions

#### normalizeServerUrl

```typescript
export function normalizeServerUrl(raw: string): string | null {
  const trimmed = raw.trim().replace(/\/$/, '');
  if (!trimmed) return null;
  if (!trimmed.startsWith('http://') && !trimmed.startsWith('https://')) {
    return null;
  }
  return trimmed;
}
```

#### getServerUrl

```typescript
export function getServerUrl(): string {
  const url = getServerUrlOrNull();
  if (!url) {
    throw new Error(
      'No server URL configured. Open Admin → Settings and enter your loon-server URL.',
    );
  }
  return url;
}
```

**Priority:**
1. `localStorage` value
2. `VITE_LOON_SERVER` environment variable

#### setServerUrl

```typescript
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
```

#### useServerUrl (React Hook)

```typescript
export function useServerUrl(): string | null {
  return useSyncExternalStore(
    subscribeServerUrl,
    getServerUrlOrNull,
    getServerUrlOrNull,
  );
}
```

**Reactive:** Updates when localStorage changes.

---

## Artwork URL Resolution

### resolveArtworkUrl

**Signature:**
```typescript
export function resolveArtworkUrl(
  path: string | undefined,
  server: string,
  cacheVersion?: string | number,
): string | undefined
```

**Purpose:** Converts relative artwork paths to absolute URLs with cache busting.

**Implementation:**
```typescript
if (!path) return undefined;

let url: string;
if (path.startsWith('http://') || path.startsWith('https://')) {
  url = path;
} else {
  url = `${server}${path.startsWith('/') ? path : `/${path}`}`;
}

if (cacheVersion !== undefined) {
  const separator = url.includes('?') ? '&' : '?';
  return `${url}${separator}v=${encodeURIComponent(String(cacheVersion))}`;
}

return url;
```

**Usage:**
```typescript
const artworkVersion = detail?.tmdb_id ?? refreshEpoch;
const posterUrl = resolveArtworkUrl(detail.poster_url, server, artworkVersion);
```

---

### streamUrl

**Signature:**
```typescript
export function streamUrl(server: string, slug: string): string
```

**Returns:** Full stream URL for video playback.

**Example:**
```typescript
streamUrl('http://192.168.1.100:3000', 'alien-1979')
// Returns: "http://192.168.1.100:3000/stream/alien-1979"
```
