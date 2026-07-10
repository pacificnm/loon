# Components Documentation

## Component Index

| Component | File | Purpose |
|-----------|------|---------|
| `AppShell` | `components/layout/AppShell.tsx` | Main layout wrapper |
| `TopBar` | `components/layout/TopBar.tsx` | Navigation header |
| `FocusButton` | `components/FocusButton.tsx` | Focusable button/tile |
| `HorizontalRow` | `components/HorizontalRow.tsx` | Browse row with horizontal scroll |
| `MovieAlphabetList` | `components/MovieAlphabetList.tsx` | A-Z movie list with index rail |
| `MovieVerticalList` | `components/MovieVerticalList.tsx` | Vertical movie list |
| `PosterCard` | `components/PosterCard.tsx` | Movie poster card |

---

## AppShell

**File:** `components/layout/AppShell.tsx`

**Purpose:** Main application layout wrapper with TopBar navigation.

### Props

```typescript
interface AppShellProps {
  focusEpoch?: number;  // Triggers focus reset on change
}
```

### Structure

```tsx
<div className={styles.shell}>
  <TopBar focusEpoch={focusEpoch} onNavigate={(path) => navigate(path)} />
  <main className={styles.main}>
    <Outlet />  {/* Rendered page content */}
  </main>
</div>
```

### Behavior

- Wraps all pages with consistent layout
- TopBar contains navigation (Home, Search, Genres, Favorites, Admin)
- Uses React Router `<Outlet>` for page rendering
- `focusEpoch` prop triggers focus reset when changed (used for page transitions)

---

## FocusButton

**File:** `components/FocusButton.tsx`

**Purpose:** Focusable button component for TV remote navigation.

### Props

```typescript
interface FocusButtonProps {
  label: string;
  focusKey?: string;      // Optional unique focus key
  selected?: boolean;     // Selected state styling
  onPress: () => void;    // Enter key handler
}
```

### Usage

```tsx
<FocusButton
  focusKey="detail-play"
  label="Play"
  onPress={() => navigate(`/play/${detail.slug}`)}
/>
```

### CSS Classes

| Class | Applied When |
|-------|--------------|
| `.button` | Always |
| `.focused` | Element has focus |
| `.selected` | `selected={true}` |

---

## FocusTile

**File:** `components/FocusButton.tsx`

**Purpose:** Focusable container for cards and tiles.

### Props

```typescript
interface FocusTileProps {
  focusKey?: string;
  className?: string;
  children: ReactNode;
  onPress?: () => void;
}
```

### Usage

```tsx
<FocusTile
  focusKey={`cast-${index}`}
  className={styles.castCard}
  onPress={() => onSelectPerson(member)}
>
  <div>...</div>
</FocusTile>
```

### Keyboard Handling

- `Enter` key triggers `onPress`
- Click also triggers `onPress` (for testing)

---

## HorizontalRow

**File:** `components/HorizontalRow.tsx`

**Purpose:** Horizontal scrolling row for browse interface (similar movies, cast, etc.).

### Props

```typescript
interface HorizontalRowProps {
  title: string;
  prefix: string;           // Unique prefix for focus keys
  movies: MovieSummary[];
  resolveArtwork: (path: string | undefined) => string | undefined;
  onSelect: (movie: MovieSummary) => void;
}
```

### Structure

```tsx
<section className={styles.section}>
  <h2 className={styles.title}>{title}</h2>
  <div className={styles.scroller}>
    <div ref={rowRef} className={styles.row}>
      {movies.map((movie) => (
        <FocusableCard
          key={movie.slug}
          prefix={prefix}
          movie={movie}
          posterUrl={resolveArtwork(movie.poster_url)}
          rowRef={rowRef}
          onSelect={onSelect}
        />
      ))}
    </div>
  </div>
</section>
```

### Focus Behavior

- Focus key: `row-{prefix}`
- First card auto-focused: `preferredChildFocusKey`
- Scroll tracking: Cards scroll into view when focused

### Scroll Into View

```typescript
onFocus: (layout) => {
  const rowRect = row.getBoundingClientRect();
  const itemLeft = layout.x;
  const itemRight = layout.x + layout.width;
  if (itemLeft < rowRect.left + 8) {
    row.scrollLeft -= rowRect.left - itemLeft + 48;
  } else if (itemRight > rowRect.right - 8) {
    row.scrollLeft += itemRight - rowRect.right + 48;
  }
}
```

---

## MovieAlphabetList

**File:** `components/MovieAlphabetList.tsx`

**Purpose:** Alphabetically grouped movie list with A-Z index rail for quick navigation.

### Props

```typescript
interface MovieAlphabetListProps {
  movies: MovieSummary[];
  server: string;
  focusEpoch?: number;
  onSelect: (movie: MovieSummary) => void;
}
```

### Structure

```tsx
<div className={styles.shell}>
  <div ref={listRef} className={styles.list}>
    {groups.map((group) => (
      <LetterBlock
        key={group.letter}
        group={group}
        server={server}
        listRef={listRef}
        onSelect={onSelect}
      />
    ))}
  </div>
  <aside className={styles.indexRail}>
    {ALPHABET_LETTERS.map((letter) => (
      activeLetters.has(letter) ? (
        <EnabledIndexLetter key={letter} letter={letter} onJump={scrollToLetter} />
      ) : (
        <DisabledIndexLetter key={letter} letter={letter} />
      )
    ))}
  </aside>
</div>
```

### Letter Grouping

```typescript
// Groups movies by first letter (A-Z, or # for non-alpha)
export function groupMoviesByLetter(movies: MovieSummary[]): LetterGroup[] {
  const groups: LetterGroup[] = [];
  for (const movie of movies) {
    const letter = letterForTitle(movie.title);
    const last = groups[groups.length - 1];
    if (last?.letter === letter) {
      last.movies.push(movie);
    } else {
      groups.push({ letter, movies: [movie] });
    }
  }
  return groups;
}
```

### Index Rail Navigation

**Enabled Letters:**
- Focusable with `useFocusable`
- Arrow navigation (up/down only)
- Enter jumps to first movie in section

**Disabled Letters:**
- Non-focusable visual placeholder
- Maintains A-Z structure

### Scroll Behavior

```typescript
const scrollToLetter = (letter: string) => {
  const list = listRef.current;
  const section = list.querySelector(`[data-letter="${letter}"]`);
  list.scrollTop += sectionRect.top - listRect.top;
  updateAllLayouts();  // Recalculate spatial navigation

  // Focus first movie in section
  const first = groups.find((g) => g.letter === letter)?.movies[0];
  if (first) {
    setFocus(itemFocusKey(first.slug));
  }
};
```

### Movie Row Component

```tsx
function MovieRow({ movie, server, listRef, onSelect }: MovieRowProps) {
  const { ref, focused } = useFocusable({
    focusKey: `movie-${movie.slug}`,
    onEnterPress: () => onSelect(movie),
    onFocus: (layout) => scrollItemIntoView(list, layout),
  });

  return (
    <article className={`${styles.row} ${focused ? styles.focused : ''}`}>
      <div className={styles.posterFrame}>
        {posterUrl ? <img src={posterUrl} /> : <div>{movie.title[0]}</div>}
      </div>
      <div className={styles.meta}>
        <h3>{movie.title}</h3>
        <p>{[movie.year, `${movie.runtime_minutes} min`].join(' · ')}</p>
        {movie.summary && <p>{movie.summary}</p>}
      </div>
    </article>
  );
}
```

---

## MovieVerticalList

**File:** `components/MovieVerticalList.tsx`

**Purpose:** Vertical list for search results and favorites.

### Props

```typescript
interface MovieVerticalListProps {
  movies: MovieSummary[];
  server: string;
  focusEpoch?: number;
  onSelect: (movie: MovieSummary) => void;
}
```

### Structure

Similar to `MovieAlphabetList` but without letter grouping or index rail.

---

## PosterCard

**File:** `components/PosterCard.tsx`

**Purpose:** Movie poster card for horizontal rows.

### Props

```typescript
interface PosterCardProps {
  movie: MovieSummary;
  posterUrl?: string;
  focused: boolean;  // From useFocusable
}
```

### Structure

```tsx
<article className={`${styles.card} ${focused ? styles.focused : ''}`}>
  <div className={styles.posterFrame}>
    {posterUrl ? (
      <img className={styles.poster} src={posterUrl} alt="" loading="lazy" />
    ) : (
      <div className={styles.placeholder}>{movie.title.slice(0, 1)}</div>
    )}
  </div>
  <h2 className={styles.title}>{movie.title}</h2>
  {movie.year && <p className={styles.year}>{movie.year}</p>}
</article>
```

### Lazy Loading

Posters use `loading="lazy"` for performance with long lists.

---

## ErrorBoundary

**File:** `components/ErrorBoundary.tsx`

**Purpose:** React error boundary for graceful error handling.

### Props

```typescript
interface ErrorBoundaryProps {
  resetKeys?: unknown[];  // Resets boundary when changed
  children: ReactNode;
}
```

### Behavior

- Catches React rendering errors
- Displays error message with retry option
- Resets on route change (`resetKeys={[pathname, search, hash]}`)

---

## ErrorFallback

**File:** `components/ErrorFallback.tsx`

**Purpose:** Error display component shown by ErrorBoundary.

### Props

```typescript
interface ErrorFallbackProps {
  error: Error;
  resetErrorBoundary: () => void;
}
```

---

## GlobalErrorShell

**File:** `components/GlobalErrorShell.tsx`

**Purpose:** Top-level error boundary for the entire app.

### Behavior

- Catches errors before React rendering
- Shows full-screen error message
- Provides reload option

---

## CSS Modules

All components use CSS Modules for scoped styling:

| Component | CSS File |
|-----------|----------|
| AppShell | `AppShell.module.css` |
| FocusButton | `FocusButton.module.css` |
| HorizontalRow | `HorizontalRow.module.css` |
| MovieAlphabetList | `MovieAlphabetList.module.css` |
| PosterCard | `PosterCard.module.css` |

### Naming Conventions

- `.card` - Base element
- `.cardFocused` - Focus state
- `.title` - Title text
- `.year` - Year text
- `.posterFrame` - Container for poster
- `.poster` - Poster image
- `.placeholder` - Fallback when no poster

---

## Focus Management

### Focus Context Setup

**File:** `main.tsx`

```tsx
// Initialize spatial navigation
init({
  debug: import.meta.env.DEV,
  visualDebug: false,
  distanceCalculationMethod: 'center',
  useGetBoundingClientRect: true,
});

// Root focus boundary
function RootFocusWrapper({ children }) {
  const { ref, focusKey } = useFocusable({
    focusable: false,
    trackChildren: true,
    isFocusBoundary: true,
    focusKey: 'root',
  });

  return (
    <FocusContext.Provider value={focusKey}>
      <div ref={ref}>{children}</div>
    </FocusContext.Provider>
  );
}
```

### Focus Keys

Naming convention: `{component}-{identifier}`

```
root
├── movie-alphabet-list
│   ├── movie-alien-1979
│   ├── movie-blade-runner-1982
│   └── index-A, index-B, ...
├── horizontal-row-similar
│   ├── similar-movie-slug-1
│   └── similar-movie-slug-2
└── cast-row
    ├── cast-0
    ├── cast-1
    └── ...
```

### Focus Epoch

Pages use `focusEpoch` to reset focus on navigation:

```tsx
const [focusEpoch, setFocusEpoch] = useState(0);

const handleRelaunch = useCallback(() => {
  setFocusEpoch((epoch) => epoch + 1);
}, []);

useWebOsLifecycle(handleRelaunch);

// Pass to pages
<AppShell focusEpoch={focusEpoch} />
```
