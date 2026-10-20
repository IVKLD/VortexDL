# VortexDL Developer Guide

This document provides a concise technical overview of the VortexDL architecture, database, pipelines, and frontend.

---

## 1. Project Structure

VortexDL is split into a **Rust Backend** and an **Angular Frontend**, compiled into a single self-contained binary at release time.

```
├── Cargo.toml                  # Backend dependencies
├── build.rs                    # Compiles Angular frontend when the web feature is enabled
├── Justfile                    # Developer task runner (install, backend, frontend, dist)
├── src/                        # Rust backend
│   ├── main.rs                 # Minimal entry point (Args parse -> app::run)
│   ├── app/                    # Application lifecycle, AppState, bootstrap & background tasks
│   ├── api/                    # REST API, WebSocket & Swagger UI handlers
│   ├── cli/                    # CLI commands & argument handling
│   ├── downloader/             # Download queue, worker pool, cancellation & finalization
│   ├── providers/              # Streaming & discovery providers (SoundCloud, YouTube)
│   ├── library/                # Music catalog scanner, metadata reader/writer & in-memory index
│   ├── devices/                # Device registry, ADB transport & incremental sync engine
│   ├── backup/                 # Settings backup & snapshot management (Local, URL, WebDAV)
│   ├── settings/               # User settings model & reactive SettingsManager
│   ├── infra/                  # Infrastructure (redb storage, HTTP client & proxy pool, logging)
│   ├── domain/                 # Core domain entities (Track, AudioFormat, Platform)
│   └── terminal/               # Terminal spinners & multi-progress bars
└── frontend/                   # Angular 21 SPA (standalone components + signals)
```

---

## 2. Backend Architecture

The backend follows a layered architecture where dependencies flow strictly downward:
Adapters (`api`, `cli`) → Application (`app`) → Orchestration (`downloader`) → Domain Services (`providers`, `library`, `devices`, `backup`, `settings`) → Infrastructure (`infra`) → Pure Leaf Types (`domain`, `terminal`).

### A. Infrastructure & Storage (`src/infra/`)
Powered by [redb](https://github.com/cberner/redb), a pure-Rust embedded key-value store. It persists user configurations and cached track file metadata. Also provides connection/proxy pools and structured logging.

### B. Library Catalog (`src/library/`)
Recursively scans music directories, parses ID3 tags and audio metadata, caches indexed files in redb, and maintains an in-memory `Library` index.

### C. Downloader Pipeline (`src/downloader/`)
Coordinates multi-stage downloads with concurrency limits, cancellation channels, and WebSocket progress events:
1.  **Queue & Discovery (`pipeline.rs`)**: Resolves remote tracks from SoundCloud / YouTube via `providers`.
2.  **Worker Pool (`worker.rs`)**: Concurrent downloads with rate-limiting and graceful cancellation signals.
3.  **Finalization (`finalize.rs`)**: Writes ID3 tags, updates cache and in-memory index, and triggers ADB device sync.

### D. Device Sync (`src/devices/`)
Monitors connected Android players/devices via ADB daemon sockets (`host:track-devices`):
1.  Scans external storage and SD card music directories.
2.  Computes diff against local library: pushes new tracks (`Artist/Track.mp3`) and removes deleted ones.
3.  Triggers Android media scanner broadcasts so tracks appear instantly in media players.

---

## 3. Frontend Architecture (`frontend/`)

Built with **Angular 21** using standalone components and Angular Signals for reactive state.

### A. Core Folders
*   `src/app/pages/`: Page components representing application views:
    *   `dashboard-view/`: Main search/download panel and active progress queue.
    *   `music-tracks-view/`: Catalog view of indexed local music with search, filter, and sorting.
    *   `settings-view/`: Configuration panel.
*   `src/app/services/`: Core business logic services:
    *   `download-tracking.service.ts`: Establishes the real-time SSE stream with backend and manages the progress state of downloading tracks.
    *   `player.service.ts`: Audio playing state manager.

### B. State Management (Signals)
All reactive states (such as active downloads queue, local library listing, search terms, and current playing tracks) are managed using Angular Signals (`signal()`, `computed()`), providing lightweight, zone-less state updates.

### C. Real-Time Communication (SSE)
`DownloadTrackingService` connects to the backend Server-Sent Events endpoint `/api/download/events` using the native browser `EventSource` API. 
The service parses JSON stream payloads pushed by Axum and dynamically updates the UI progress bars or pushes newly completed tracks straight into the local tracks signal.

---

## 4. Developer Commands Cheatsheet

VortexDL relies on a `Justfile` to coordinate commands:

```bash
# Install dependencies for both Rust and Angular
just install

# Start backend in watch-reload mode (REST API on http://localhost:3200)
just backend

# Start frontend development server (with hot reload on http://localhost:4200)
just frontend

# Format backend and frontend files
just fmt

# Lint codebases
just lint

# Build production executable with embedded frontend (located in target/release/vortex-dl)
just dist
```
