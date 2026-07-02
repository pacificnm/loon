# Loon Client — Focus & Navigation

Remote-first spatial navigation for Magic Remote / D-pad. See [README.md](README.md) for principles.

**Rule:** Every screen works with **Arrow keys, OK, and Back only.**

---

## Decisions

| Topic | Decision |
|-------|----------|
| Focus library | **@noriginmedia/norigin-spatial-navigation** for W0 spike; custom fallback if TV checklist fails |
| Hover | None — `:focus` styles only |
| Focus memory | Restore row index + card index when returning from detail/player |
| Back stack | React Router history + `focusMemory` in location state |

---

## Global key handling

| Key | webOS keyCode | Global behavior |
|-----|---------------|-----------------|
| Up | 38 | Spatial move up |
| Down | 40 | Spatial move down |
| Left | 37 | Spatial move left |
| Right | 39 | Spatial move right |
| OK | 13 | Activate focused element |
| Back | 461 | See [Back behavior](#back-behavior) |

Register `keydown` on `window` in capture phase. Prevent default for arrow keys except in text inputs.

### Back behavior

| Context | First Back | Second Back |
|---------|------------|-------------|
| Side rail open | Close rail | — |
| Player — overlay visible | Hide overlay | — |
| Player — no overlay | Exit player (save progress) | — |
| Detail / Search / Movies | Pop route (restore focus) | — |
| Home | Open side rail | Exit app (webOS) |
| Side rail closed on Home | — | `webOS.platformBack()` if available |
| Settings (root) | Home | Exit app |

---

## Focus tree by screen

### Home

```text
FocusLayer: top-bar
  [Search] — [Settings]

FocusLayer: hero (optional)
  [Watch]

FocusLayer: row-0 (Continue Watching)
  [card-0] [card-1] [card-2] ...

FocusLayer: row-1 (Recently Added)
  [card-0] [card-1] ...

... row-N
```

**Entry focus:** `row-0.card-0` (first card of first row). If no continue watching, first card of recently added.

**Vertical moves:** Jump between layers. When entering a row, preserve horizontal index clamped to row length.

**Horizontal moves:** Scroll row container so focused card is in the **center 60%** of the viewport.

**OK:** Navigate to `/movie/:slug` with state `{ focus: { row, col } }`.

### Movies grid

```text
FocusLayer: header
  [Sort]

FocusLayer: grid
  row-major cells [0,0] … [5,0], [0,1] …
```

6 columns at 1080p. Right on last column of a row does nothing (no wrap). Down on last row triggers pagination fetch.

### Movie detail

```text
FocusLayer: actions
  [Resume?] [Play] [More Info]

FocusLayer: favorite
  [♥]

FocusLayer: cast (when More Info expanded)
  [cast-0] [cast-1] … horizontal
```

**Entry focus:** Resume or Play.

**Vertical:** Actions → cast row when expanded. Up from actions → favorite.

### Search

```text
FocusLayer: input
  [search field]

FocusLayer: results
  [card-0] [card-1] …
```

**Entry focus:** Input field. Down moves to results when `movies.length > 0`.

### Settings

```text
FocusLayer: form
  [Server URL field]
  [Test Connection]
  (About is static — not focusable)
```

### Player

Two modes: **idle** (video only) and **controls** (overlay focused).

```text
Mode: idle
  — arrows handled by player (seek L/R, show overlays U/D)

Mode: controls-visible
  FocusLayer: transport
    [⏪] [▶] [⏩] [CC hidden] [Audio hidden]
  FocusLayer: scrubber
    [progress bar]
```

**Entry:** idle mode, no focus ring (immersive). Down → controls mode, focus Play. Auto-hide controls after 5s idle.

### Side rail

```text
FocusLayer: menu
  [Home] [Movies] [Search] [Settings] [Server]
```

**Entry focus:** Current section highlighted. OK → navigate + close rail.

---

## Focus memory

When navigating `Home → Detail → Back`:

```typescript
interface FocusMemory {
  screen: 'home' | 'movies' | 'search';
  rowIndex?: number;
  colIndex?: number;
  slug?: string; // scroll-into-view hint
}
```

Pass via React Router `location.state`. On Back, Home reads state and calls `setFocus(rowIndex, colIndex)` after data loads.

Player return: same memory — do not reset to top of home.

---

## Scroll & animation

| Interaction | Animation |
|-------------|-----------|
| Card focus | `transform: scale(1.08)` + `box-shadow` glow, 180ms ease-out |
| Row scroll | `scroll-behavior: smooth`, max 300ms |
| Hero parallax | Subtle backdrop `scale(1.05)` on hero focus — optional W3 |
| Side rail | `translateX` slide 200ms |
| Screen transition | Cross-fade 150ms (no slide between routes — TV apps feel heavy with slides) |

---

## Focus ring style

```css
/* Applied via .focused class from spatial nav */
.focused {
  outline: 3px solid #38BDF8;
  outline-offset: 4px;
  box-shadow: 0 0 20px rgba(56, 189, 248, 0.45);
}
```

Poster cards use outline on the card container, not the image (prevents clip).

---

## Accessibility notes

- webOS TalkBack: defer post-v1
- Minimum contrast: text on `#141821` ≥ 4.5:1 (muted text is large enough for 3:1)
- Never rely on color alone — favorite uses filled vs outline heart

---

## W0 spike checklist (focus)

Before committing to norigin:

- [ ] Focus visible on 1080p TV from 10 feet
- [ ] No focus trap in player idle mode
- [ ] Back always escapes current context predictably
- [ ] Row scroll keeps focused card on screen
- [ ] Virtual keyboard in Search does not break spatial tree

If any fail after one fix attempt → prototype minimal custom `FocusManager` in `src/focus/spatial.ts`.
