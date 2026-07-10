# Loon

> A lightweight, self-hosted movie streaming server and webOS client built with Rust and the Nest Framework.

## Vision

Loon is a personal movie streaming system designed specifically for LG webOS televisions.

The project consists of:

* **Loon Server** — A Rust application running on a home server that manages the movie library and streams media.
* **Loon webOS** — A native LG webOS application built with React that provides a beautiful, remote-friendly user experience.
* **Loon Admin Desktop** — A Tauri + React admin app for library management, TMDB matching, and server settings.

Loon is intentionally designed for a single platform. Rather than attempting to support every Smart TV, browser, or mobile device, it focuses entirely on delivering the best possible experience for LG webOS.

---

# Goals

* Beautiful Netflix-inspired interface
* Extremely fast browsing
* Instant playback
* Lightweight server
* Modern Rust architecture
* Self-hosted
* API-first backend
* Native LG webOS experience

---

# Non-Goals

Loon is **not** intended to become another Plex, Kodi, or Jellyfin.

The project intentionally excludes:

* Live TV
* DVR
* IPTV
* Music
* Photos
* Plugin systems
* Mobile applications
* Desktop video clients (playback is webOS only)
* Browser clients
* Multiple TV platforms
* Docker / container deployment

The only supported **video client** is **LG webOS**.

A **desktop admin app** (Tauri + React) exists for library management, TMDB matching, and server settings — but not for watching movies.

---

# Design Principles

## LG webOS First

Every design decision should optimize the experience for LG televisions.

The interface should be designed for:

* Large displays
* TV remotes
* 10-foot viewing distance
* Fast navigation
* Minimal clicks

Keyboard and mouse interaction are not design goals.

---

## Performance First

Movies should stream directly whenever possible.

Avoid unnecessary transcoding.

Background processing should never impact playback.

---

## Simple Architecture

```text
LG webOS App
        │
HTTP API
        │
Loon Server
        │
Movie Library
```

There are only two applications.

---

## API First

The webOS application communicates exclusively through HTTP APIs.

The frontend contains presentation logic only.

All media management lives on the server.

---

## Modular Backend

Loon is built using reusable Nest modules.

Example Nest crates Loon composes:

* [`nest-http-serve`](../../docs/nest-http-serve/README.md) — HTTP host
* [`nest-file`](../../docs/nest-file/README.md) — scoped filesystem I/O
* [`nest-media`](../../docs/nest-media/README.md) — media types + provider traits
* [`nest-media-library`](../../docs/nest-media-library/README.md) — library scan/index
* [`nest-tmdb`](../../docs/nest-tmdb/README.md) — TMDB metadata
* [`nest-transcode`](../../docs/nest-transcode/README.md) — FFprobe inspection
* [`nest-config`](../../docs/nest-config/README.md) — TOML configuration
* [`nest-data-sqlite`](../../docs/nest-data-sqlite/README.md) — SQLite (v0.2 catalog)

Planned extractions: [`nest-stream`](../../docs/plan/nest-stream-v1.md), [`nest-cache`](../../docs/plan/nest-cache-v1.md).

Application-specific logic (routes, slugs, catalog, webOS UX) stays in the Loon repo.

---

# User Experience

The experience should resemble modern streaming services.

Features include:

* Hero banner
* Large movie artwork
* Genre rows
* Continue Watching
* Recently Added
* Instant search
* Movie details
* Resume playback

Configuration should be minimal.

Users should spend their time watching movies, not configuring software.

---

# Architecture

```text
Loon webOS
(Vite + React)

        │

HTTP JSON API

        │

Loon Server
(Rust)

        │

Movie Library
```

---

# Long-Term Vision

Loon exists to provide the best possible personal movie streaming experience on LG webOS televisions.

Instead of supporting every device and feature imaginable, Loon embraces a focused philosophy:

**Do one thing exceptionally well.**

---

# Documentation

### Product

- [README](../README.md) — vision, non-goals, UX goals

### Server

- [API reference](docs/api.md) — implemented routes (v0.1)
- [API roadmap](docs/api-roadmap.md) — **finish server API before UI**
- [v1 plan](docs/v1.md) — phases, stream, data model, ops
- [api-v0.2](docs/api-v0.2.md) — browse, search, progress, favorites (spec)
- [data-v1](data-v1.md) — SQLite repository, migrations
- [repo-v1](repo-v1.md) — Git repo layout, CI, Cargo workspace
- [setup-v1](setup-v1.md) — native install, systemd, first-run
- [implementation-v1](implementation-v1.md) — build checklist

### webOS client

- [webos-v1](webos-v1.md) — screens, player, packaging
- [webos-test-checklist](webos-test-checklist.md) — manual LG TV tests

### Desktop Admin

- [desktop/README.md](desktop/README.md) — Tauri + React admin app
- [desktop/docs/v1.md](desktop/docs/v1.md) — implementation plan

### Config

- [config.example.toml](../config.example.toml)
