# Loon Client — W0 Spike Plan

**Goal:** Prove the full loop on a **real LG webOS TV** — fetch movies from `loon-server`, navigate with D-pad, play a stream.

**Exit criteria:** Checklist sections **A + B** in [webos-test-checklist.md](../docs/webos-test-checklist.md) pass on hardware.

**Duration:** 1–2 days (broken into tasks below).

**Not in W0:** React Router, Settings UI, browse rows, hero, progress API, side rail, TanStack Query (optional stub).

---

## Prerequisites

Complete before Day 1:

| # | Task | How to verify |
|---|------|---------------|
| P1 | `loon-server` running on LAN | `curl http://SERVER:3000/api/health` → `"status":"ok"` |
| P2 | Library scanned, movies in catalog | `curl http://SERVER:3000/api/movies` → non-empty |
| P3 | At least one **MP4** movie (H.264) | Note slug for test movie |
| P4 | webOS CLI installed on dev machine | `ares-setup-device` lists TV |
| P5 | TV and server on same subnet | TV browser can reach server IP |
| P6 | Read [FOCUS.md](FOCUS.md) + [LG Design Principles](https://webostv.developer.lge.com/develop/guides/design-principles) | — |

**Dev fallback:** `VITE_LOON_SERVER=http://192.168.x.x:3000` in `client/.env.local` so W0 skips Settings UI.

---

## Day 1 — Scaffold, API, browser proof

### Morning (2–3 h): Project scaffold

| # | Task | Done when |
|---|------|-----------|
| 1.1 | Create `client/` with Vite + React + TypeScript (`npm create vite@latest`) | `npm run dev` serves blank app |
| 1.2 | Add folder layout from [README.md](README.md) — minimal: `src/main.tsx`, `src/App.tsx`, `src/config.ts`, `src/api/client.ts`, `src/api/types.ts`, `src/theme/tokens.css` | Structure matches plan |
| 1.3 | Set `vite.config.ts` → `base: './'` (required for webOS `.ipk`) | Build output uses relative asset paths |
| 1.4 | Add `appinfo.json` draft from [TECH.md](TECH.md) + placeholder icons | File present in `client/` |
| 1.5 | Add `.gitignore` (`node_modules`, `dist`, `.env.local`) | — |
| 1.6 | Wire dark theme tokens (`#090B10` background) | App renders dark full-screen |

**Checkpoint:** `npm run build` succeeds; `dist/index.html` opens in desktop browser.

---

### Midday (2–3 h): API client + movie list

| # | Task | Done when |
|---|------|-----------|
| 2.1 | Implement `getServerUrl()` — env var only for W0 | `config.ts` returns URL |
| 2.2 | Implement `LoonApi.fetchMovies()` → `GET /api/movies` | Returns typed `MovieSummary[]` |
| 2.3 | Parse server error envelope `{ error: { code, message } }` | Throws readable error |
| 2.4 | Build single horizontal row of `PosterCard` (static CSS, no spatial lib yet) | Posters render from `poster_url` |
| 2.5 | Prepend server base URL to relative artwork paths (`/api/artwork/...`) | Images load on LAN |
| 2.6 | Loading state (skeleton cards) + error banner on fetch fail | — |

**Checkpoint:** Desktop browser at `npm run dev` shows movie row from real server.

**Test commands:**

```bash
curl -s http://SERVER:3000/api/movies | jq '.movies | length'
curl -s -o /dev/null -w "%{http_code}" http://SERVER:3000/api/artwork/SLUG/poster
```

---

### Afternoon (2–3 h): Spatial focus + selection

| # | Task | Done when |
|---|------|-----------|
| 3.1 | Install `@noriginmedia/norigin-spatial-navigation` | Package in `package.json` |
| 3.2 | Wrap app in `FocusContextProvider` | — |
| 3.3 | Make `PosterCard` focusable; apply focus ring from [FOCUS.md](FOCUS.md) | Scale 1.08× + `#38BDF8` outline |
| 3.4 | Left/Right moves between cards in row | Focus visible in browser (arrow keys) |
| 3.5 | OK on card sets `selectedSlug` state | — |
| 3.6 | Log focus issues; note if norigin needs `useFocusable` per card | — |

**Checkpoint:** Arrow keys move focus across posters in desktop Chrome.

**W0 norigin spike checklist** (from [FOCUS.md](FOCUS.md)):

- [ ] Focus ring visible
- [ ] No focus trap
- [ ] Left/Right scroll row into view (basic `scrollIntoView`)

If norigin fails in browser after one fix attempt, stub `src/focus/spatial.ts` with manual `focusedIndex` + keydown handler and continue.

---

## Day 2 — Player, webOS package, TV proof

### Morning (2–3 h): Video player

| # | Task | Done when |
|---|------|-----------|
| 4.1 | Add `VideoPlayer` component — full-screen `<video>` | Renders black full viewport |
| 4.2 | On OK / select: `video.src = {server}/stream/{slug}`; `video.play()` | Plays in desktop browser |
| 4.3 | Back (keydown 461 or Escape in browser) stops video, returns to row | — |
| 4.4 | Handle `onError` — show message if stream fails | — |
| 4.5 | W0 player: no overlay UI, no progress API | Intentionally minimal |

**Checkpoint:** Desktop browser plays movie from `loon-server` stream URL.

**Test:**

```bash
curl -I -H "Range: bytes=0-1" http://SERVER:3000/stream/SLUG
# Expect 206 Partial Content
```

---

### Midday (1–2 h): webOS build pipeline

| # | Task | Done when |
|---|------|-----------|
| 5.1 | `npm run build` → copy output to package root layout webOS expects | `dist/` has `index.html` + assets |
| 5.2 | Place `appinfo.json` + icons in package root | — |
| 5.3 | `ares-package ./dist` (or documented folder) → `.ipk` | Package builds without error |
| 5.4 | Add `package:webos` script to `package.json` | One command builds + packages |
| 5.5 | Document device name in `client/README` or comment in script | — |

**Checkpoint:** `.ipk` file exists on disk.

---

### Afternoon (2–4 h): Install on TV + fix loop

| # | Task | Done when |
|---|------|-----------|
| 6.1 | `ares-install --device TV com.pacificnm.loon_*.ipk` | App appears on TV |
| 6.2 | Launch app — movie row loads | Posters visible |
| 6.3 | D-pad Left/Right — focus moves | 10-foot visible ring |
| 6.4 | OK — video plays within 5s | Audio + picture |
| 6.5 | Back — returns to row | — |
| 6.6 | `ares-inspect --device TV` — debug if needed | Can see console errors |
| 6.7 | Fix TV-specific issues (CORS, `base` path, codec, focus) | Checklist B passes |

**Common TV failures:**

| Symptom | Likely fix |
|---------|------------|
| Blank screen | `base: './'` in Vite; check `index.html` script paths |
| Images 404 | Prepend server URL to relative `poster_url` |
| Stream won't play | Use MP4/H.264 test file; check Range support |
| Focus invisible | Increase outline width; check webOS overscan safe margin 48px |
| CORS error | Server `[http] cors_origins` includes `*` or TV origin |

**Checkpoint:** **W0 complete** — sections A + B of manual checklist signed off.

---

## W0 deliverables

| Artifact | Location |
|----------|----------|
| Runnable webOS app | `client/dist/` + `.ipk` |
| Minimal source | `client/src/` — config, api, PosterCard, VideoPlayer, App |
| Build script | `npm run package:webos` |
| Spike notes | Update failures table in [webos-test-checklist.md](../docs/webos-test-checklist.md) |

---

## W0 → W1 handoff

Do **not** start W1 until W0 exit criteria pass. W1 adds:

- React Router (Home → Movie Detail → Player)
- Settings screen + `localStorage` server URL
- Proper error / offline screens
- Remove hard-coded `VITE_LOON_SERVER` dependency

See [README.md](README.md) implementation phases.

---

## Time budget summary

| Block | Hours | Cumulative |
|-------|-------|------------|
| Day 1 AM — scaffold | 2–3 | 3 |
| Day 1 midday — API + row | 2–3 | 6 |
| Day 1 PM — spatial focus | 2–3 | 9 |
| Day 2 AM — player | 2–3 | 12 |
| Day 2 midday — package | 1–2 | 14 |
| Day 2 PM — TV debug | 2–4 | 18 |

**Realistic:** 1 focused day if server + TV are already set up; 2 days with toolchain learning or codec issues.

---

## Optional stretch (still W0 if time permits)

- [ ] Second row from `GET /api/browse` instead of flat movies list
- [ ] `npm test` with one API client unit test (Vitest + mock fetch)
- [ ] CI job: `npm run build` in loon GitHub Actions

Do not expand scope into W1 features (detail screen, router) during the spike.

---

## Related

- [SCREENS.md](SCREENS.md) — full screen specs (W1+)
- [COMPONENTS.md](COMPONENTS.md) — PosterCard dimensions
- [TECH.md](TECH.md) — dependencies and appinfo.json
- [webos-test-checklist.md](../docs/webos-test-checklist.md) — sign-off checklist
