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
│   ├── main.rs                 # Entry point, system startup, Axum runner
│   ├── cli.rs                  # CLI argument parsing
│   ├── settings.rs             # Configuration manager (memory cache + database writeback)
│   ├── storage.rs              # File scanner & metadata database catalog
│   ├── database/               # Redb embedded database layer (settings, sync)
│   ├── api/                    # REST API & SSE (Server-Sent Events) handlers
│   ├── downloader/             # Track discovery & staged download pipeline
│   └── adb_device/             # ADB Android synchronizer
└── frontend/                   # Angular 21 SPA (standalone components + signals)
```

---

## 2. Backend Architecture

### A. Database (`src/database/`)
Powered by [redb](https://github.com/cberner/redb), a pure-Rust embedded key-value store. It stores application configurations and tracks sync checkpoints for ADB-connected devices.

### B. Music Storage Scanner (`src/storage.rs`)
Iterates over the configured output directory, extracts metadata from filenames (`Artist - Title`), and maps them in the database. Serves as the source of truth for the local track list, preventing redownloading of existing tracks.

### C. Downloader Pipeline (`src/downloader/`)
Async download routines are handled inside `src/downloader/core/pipeline/` through a staged pipeline:
1.  **Prepare (`prepare.rs`)**: Initializes CLI progress bars and adds the task to `DownloadManager` with status `Downloading`.
2.  **Resolve (`resolve.rs`)**: Uses `soundcloud-rs` to fetch progressive MP3 or HLS streams from SoundCloud API.
3.  **Download (`download.rs`)**: Reads the stream asynchronously, writes chunks to disk, and pushes real-time progress to `DownloadManager`.
4.  **Complete (`complete.rs`)**: blocking task injecting ID3 tags (artist, title, cover art) into the MP3 file, transitioning the state to `Finished` and indexing it in `MusicStorage`.

### D. ADB Android Sync (`src/adb_device/`)
Runs an async timer every 3 seconds checking for connected ADB devices. On match:
1.  Fetches file list from target device directory.
2.  Deletes **orphaned tracks** that were deleted locally.
3.  Pushes **new tracks** sorted into `Artist/Track.mp3` directories.
4.  Runs native `am broadcast ...` media scan on the Android device so tracks appear instantly in players.

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
