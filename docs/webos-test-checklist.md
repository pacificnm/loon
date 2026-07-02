# Loon webOS manual test checklist

## Status: Living document

Run on a **real LG webOS TV** on the same LAN as `loon-server`. Check off after each release candidate.

Client source and UI plan: [`client/`](../client/README.md). Server setup: [setup-v1.md](setup-v1.md).

Record TV model and webOS version for every test run.

### Test run metadata

| Field | Value |
|-------|-------|
| Date | |
| Loon server version | `curl SERVER/api/health → .version` |
| Loon client version | `appinfo.json` version |
| TV model | |
| webOS version | |
| Server URL | `http://___:3000` |
| Client phase tested | W0 / W1 / W2 / W3 |

---

## A. Server reachability (before opening app)

Run from a PC on the same LAN as the TV.

- [ ] `curl http://SERVER:3000/api/health` → `"status":"ok"`
- [ ] `curl http://SERVER:3000/api/browse` → `rows` array (v0.2) **or** `curl .../api/movies` → movies (W0 minimum)
- [ ] `curl -I -H "Range: bytes=0-1" http://SERVER:3000/stream/SLUG` → `206` for a known movie
- [ ] `curl -I http://SERVER:3000/api/artwork/SLUG/poster` → `200` or `302`
- [ ] Firewall allows TV → server TCP port 3000

---

## B. W0 spike — Home row + play

**Scope:** Single movie row, D-pad focus, full-screen player. See [W0-SPIKE.md](../client/W0-SPIKE.md).

**Screens:** Home (minimal) → Player

| # | Step | Pass |
|---|------|------|
| B1 | App launches without crash | [ ] |
| B2 | Movie posters load (or placeholder if no TMDB) | [ ] |
| B3 | **D-pad Left/Right** moves focus between cards | [ ] |
| B4 | Focus ring visible at **10-foot** distance | [ ] |
| B5 | **OK** on focused card starts playback | [ ] |
| B6 | Video starts within **5 seconds** | [ ] |
| B7 | Audio plays | [ ] |
| B8 | Picture fills screen (no wrong aspect crop) | [ ] |
| B9 | **Back** exits player, returns to row | [ ] |

**API exercised:** `GET /api/movies`, `GET /stream/:slug`, artwork proxy via `poster_url`

---

## C. W1 — Core navigation

**Scope:** Router, Settings, Movie Detail, error states. No browse rows yet.

**Screens:** Bootstrap/Settings → Home → Movie Detail → Player

### C1. Bootstrap & Settings

- [ ] First launch with no saved URL → Settings / connect screen
- [ ] Enter server URL → **Test & Save** → `GET /api/health` succeeds
- [ ] URL persists in `localStorage` after app restart
- [ ] Invalid URL shows inline error (not silent fail)

### C2. Home (W1 — flat or single row)

- [ ] Home loads movie data after settings saved
- [ ] Loading skeleton shown during fetch
- [ ] Server unreachable → error screen with retry + Settings link

### C3. Movie Detail

- [ ] **OK** on card opens **Movie Detail** (not direct play)
- [ ] Detail shows title, year, summary, poster
- [ ] Backdrop fanart visible (or poster fallback)
- [ ] **Play** is default focus
- [ ] **Back** returns to Home with focus restored on same card

### C4. Player (W1)

- [ ] **Play** on detail starts stream
- [ ] **Back** exits player → detail → Home
- [ ] Error toast/message if stream 404 or network drops

**API exercised:** `GET /api/movies/:slug`, `GET /stream/:slug`

---

## D. W2 — Full v1 UX

**Scope:** Browse feed, hero, search, progress, favorites. Requires server API v0.2.

**Screens:** Home (full) → Movie Detail → Player · Search · Settings

### D1. Home

- [ ] `GET /api/browse` — **Hero** banner shows backdrop + title
- [ ] **Continue Watching** row after partial watch
- [ ] Progress bar on continue-watching poster card
- [ ] **Recently Added** row order matches server
- [ ] **Favorites** row after marking favorite on detail
- [ ] **Genre** rows appear (≥3 movies per genre)
- [ ] **Up/Down** moves between rows; **Left/Right** within row
- [ ] Row scrolls to keep focused card centered
- [ ] **Up** from first row focuses hero **Watch**
- [ ] Top bar: **Search** and **Settings** reachable via focus

### D2. Movie Detail (W2)

- [ ] **Resume** appears when `watch_progress_seconds > 0`
- [ ] **Resume** starts at saved position
- [ ] **Play** starts from beginning
- [ ] **Trailer** button not shown
- [ ] **♥ Favorite** toggles; `PUT /api/movies/:slug/favorite`
- [ ] **More Info** expands cast + full summary on same screen

### D3. Search

- [ ] Search screen opens from top bar or side rail
- [ ] Query min 2 chars → `GET /api/search?q=`
- [ ] Results navigate with D-pad
- [ ] **OK** on result → Movie Detail
- [ ] **Back** returns to previous screen
- [ ] Recent searches persist (max 5)

### D4. Settings (W2)

- [ ] Server URL editable and persists
- [ ] Status line shows version + `movies_count` from health
- [ ] **Test Connection** re-fetches health
- [ ] Read-only last scan time from `GET /api/library/status`
- [ ] No scan trigger button (admin is server-side)

### D5. Playback & progress

- [ ] **Down** shows player controls overlay
- [ ] **OK** pause/play
- [ ] **Left/Right** seek ±10s (±30s when scrubber focused)
- [ ] **Up** shows title/info overlay
- [ ] Progress saved on pause, Back, every 30s — `PUT /api/movies/:slug/progress`
- [ ] Exit mid-movie → **Continue Watching** row on Home after relaunch
- [ ] Watch >90% → movie leaves Continue Watching

**API exercised:** `GET /api/browse`, `GET /api/search`, `PUT /progress`, `PUT /favorite`, `GET /api/library/status`

---

## E. W3 — Polish

**Scope:** Side rail, animations, focus memory, Movies grid.

**Screens:** Side Rail · Movies (full grid)

### E1. Side rail

- [ ] **Back** on Home opens side rail (first press)
- [ ] Rail items: Home, Movies, Search, Settings, Server
- [ ] **TV Shows** not shown
- [ ] **OK** navigates and closes rail
- [ ] **Back** or **Right** dismisses rail
- [ ] Content dims behind rail

### E2. Movies screen

- [ ] Side rail **Movies** → paginated grid (`GET /api/movies?page=`)
- [ ] 6-column grid at 1080p; D-pad 2D navigation
- [ ] Load more when focus nears last row
- [ ] Sort control cycles title / year / recently_added

### E3. Motion & focus polish

- [ ] Card focus scale animation (~180ms)
- [ ] Screen transitions cross-fade (~150ms)
- [ ] Focus memory: return from detail/player restores row + card index
- [ ] No hover-only UI anywhere

---

## F. Stress & edge cases (any phase ≥ W1)

- [ ] Library **100+ movies** — Home scroll performance acceptable
- [ ] Movie with **no poster** — card still usable (placeholder)
- [ ] Very long title — truncated to 2 lines with ellipsis
- [ ] **MP4 H.264** plays (primary format)
- [ ] **MKV** plays — note pass/fail (codec/TV dependent)
- [ ] `503 library_scanning` → scanning screen, then Home when idle
- [ ] Empty library → empty state message

---

## G. Regression smoke (every release)

- [ ] Cold start → Home loads within 3s
- [ ] Play one movie end-to-end (detail → player → back)
- [ ] Search one movie end-to-end
- [ ] No crash after 30 min idle on Home
- [ ] Reinstall `.ipk` over previous version without clearing data

---

## Phase sign-off

| Phase | Sections | Signed off | Date |
|-------|----------|------------|------|
| W0 spike | A, B | | |
| W1 core nav | A, B, C, G | | |
| W2 full v1 | A–D, G | | |
| W3 polish | A–G | | |

---

## Failures log

| # | Phase | Screen | Step | Expected | Actual | Notes |
|---|-------|--------|------|----------|--------|-------|
| 1 | | | | | | |

---

## Related

- [client/README.md](../client/README.md) — UI plan overview
- [client/W0-SPIKE.md](../client/W0-SPIKE.md) — spike task breakdown
- [client/SCREENS.md](../client/SCREENS.md) — per-screen specs
- [client/FOCUS.md](../client/FOCUS.md) — remote key map
- [api-v0.2.md](api-v0.2.md) — server routes
