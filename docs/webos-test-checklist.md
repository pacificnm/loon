# Loon webOS manual test checklist

## Status: Living document

Run on a **real LG webOS TV** on the same LAN as `loon-server`. Check off after each release candidate.

Server must be running per [setup-v1.md](setup-v1.md). Record TV model and webOS version for each test run.

**Test run metadata**

| Field | Value |
|-------|-------|
| Date | |
| Loon server version | |
| webOS app version | |
| TV model | |
| webOS version | |
| Server URL | `http://___:3000` |

---

## A. Server reachability (before opening app)

- [ ] `curl http://SERVER:3000/api/health` returns `"status":"ok"` from a PC on LAN
- [ ] `curl http://SERVER:3000/api/movies` returns movie list
- [ ] Firewall allows TV → server port 3000

---

## B. Phase W0 — Grid + play

- [ ] App launches without crash
- [ ] Movie posters load (or placeholders if no TMDB)
- [ ] Grid scrolls with **D-pad** (up/down/left/right)
- [ ] Focus ring visible at 10-foot distance
- [ ] Select movie opens detail or plays
- [ ] Video starts within 5 seconds
- [ ] Audio plays
- [ ] Picture fills screen correctly (no wrong aspect crop)

---

## C. Phase W1 — Navigation

- [ ] Back button returns to previous screen
- [ ] Detail screen shows title, year, summary
- [ ] Play button starts stream
- [ ] Error state shown when server unreachable (pull network)

---

## D. Playback

- [ ] Seek forward 30s (if player UI supports) — stream resumes
- [ ] Seek backward 30s
- [ ] Pause and resume
- [ ] Exit player mid-movie — returns to browse
- [ ] Replay same movie — starts from beginning (v0.1) or offers resume (v0.2)

---

## E. Phase W2 — Home UX (requires API v0.2)

- [ ] Hero banner displays backdrop
- [ ] Continue Watching row appears after partial watch
- [ ] Progress saved — row shows movie after exit and relaunch app
- [ ] Recently Added row matches server order
- [ ] Favorites row after marking favorite on detail
- [ ] Genre row opens row drill-down
- [ ] Search finds movie by partial title
- [ ] Settings: change server URL persists after app restart

---

## F. Stress / edge cases

- [ ] Library with 100+ movies — grid scroll performance acceptable
- [ ] Movie with no poster — card still usable
- [ ] Very long title — truncated cleanly
- [ ] MKV file plays (note pass/fail — codec dependent)
- [ ] MP4 H.264 plays

---

## G. Regression smoke (any release)

- [ ] Cold start app → grid loads
- [ ] Play one movie end-to-end
- [ ] No memory crash after 30 min idle on home screen

---

## Failures log

| # | Step | Expected | Actual | Notes |
|---|------|----------|--------|-------|
| 1 | | | | |

---

## Related

- [webos-v1.md](webos-v1.md) — client plan
- [api-v0.2.md](api-v0.2.md) — server routes for W2
