<p align="center">
  <h2 align="center">VortexDL — Web Interface</h2>
</p>

<p align="center">
  Angular Single Page Application for library management, multi-platform search, and audio streaming.
  <br>
  <img src="https://img.shields.io/badge/Angular-v22-DD0031?style=flat-square&logo=angular&logoColor=white" alt="Angular">
  <img src="https://img.shields.io/badge/TypeScript-v6-3178C6?style=flat-square&logo=typescript&logoColor=white" alt="TypeScript">
  <img src="https://img.shields.io/badge/Vitest-Unit_Testing-6E9F18?style=flat-square&logo=vitest" alt="Vitest">
</p>

---

## Overview

The VortexDL frontend provides a web interface for the VortexDL core server. It features search across SoundCloud and YouTube, an interactive music player with Media Session integration, ADB device synchronization management, and library visualization.

## Features

- **State Management**: Reactive state management utilizing Angular Signals and RxAngular.
- **Virtual Scrolling**: Rendering of large music collections via `@angular/cdk/scrolling`.
- **Multi-Platform Discovery**: Search interface supporting both SoundCloud and YouTube tracks with live audio previews.
- **Web Audio Player**: Global persistent audio player with Media Session API integration (lock screen metadata and controls), Fisher-Yates shuffle, and logarithmic volume.
- **Client-Side Fuzzy Search**: Fast track search and filtering powered by `Fuse.js`.
- **ADB Device Management**: Interface for monitoring connected Android devices, setting target music directories, and triggering synchronization.
- **Analytics Dashboard**: Overview of library statistics and format distributions.

---

## Directory Structure

```text
frontend/src/
├── app/
│   ├── pages/
│   │   ├── dashboard/                # Statistics & activity breakdown charts
│   │   ├── music-tracks-view/        # Library management with virtual scroll
│   │   ├── search-view/              # Multi-provider music discovery (YouTube / SoundCloud)
│   │   │   └── components/search-toolbar/  # Filter tabs & search input
│   │   └── settings-view/            # Network, ADB, backup, and storage settings
│   ├── services/
│   │   ├── player.service.ts         # Global audio streaming & player state
│   │   ├── tracks.service.ts         # Local tracks CRUD operations
│   │   └── adb.service.ts            # Device tracking & synchronization API
│   └── app.component.ts              # Root navigation & persistent player bar
└── shared/                           # Reusable UI components, dialogs, and models
```

---

## Development & Build Commands

### Start Development Server

```bash
yarn start
```
Starts local dev server at `http://localhost:4200/` with live reload.

### Build Production Bundle

```bash
yarn build
```
Compiles optimized distribution bundle into `dist/voltexdl`.

### Run Unit Tests

```bash
yarn test
```
Executes unit tests using [Vitest](https://vitest.dev/).

### Lint Codebase

```bash
yarn lint
```

---

## License

GPL-3.0-only License.
