<p align="center">
  <img src="assets/logo.svg" alt="VortexDL Logo" width="128">
</p>

<h3 align="center">VortexDL</h3>

<p align="center">
  Multi-threaded SoundCloud music downloader with a sleek web panel.
  <br>
  <a href="https://github.com/IVKLD/VortexDL/releases">
    <img src="https://img.shields.io/github/v/release/IVKLD/VortexDL?style=flat-square" alt="Release">
  </a>
  <img src="https://img.shields.io/badge/License-GPLv3-blue.svg?style=flat-square" alt="License">
  <img src="https://img.shields.io/badge/Rust-stable-orange?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Angular-v21-DD0031?style=flat-square&logo=angular&logoColor=white" alt="Angular">
</p>


## Motivation
Streaming services are a piece of shit with subscriptions and censorship, and modern phones without a 3.5mm jack are a technological failure. This project was written to download music in one click and listen to it like a human being on a proper player (e.g., **Cayin N3 Ultra**).

Read the full thoughts on this — [at the bottom of the page](#manifesto).

## How it works

*   **Rust Backend:** Handles all the main work. It uses the `soundcloud-rs` library to access the API and downloads music in multiple threads so you don't have to wait forever.
*   **Progress Display:** You can see the download progress everywhere: both in the console (CLI) and in the web panel. You'll always know exactly how much is left.
*   **Web Panel:** Angular-based web interface for those who don't want to mess with the console. Just paste the link and manage your downloads visually.
*   **Single Binary:** Everything is packed into one file, so there's no need to fuck around with installing dozens of libraries and dependencies.

## Features

*   **Multi-threaded Concurrent Downloading:** Instantly downloads tracks from SoundCloud using multiple threads (configurable limits).
*   **ADB Synchronization for Android Devices:**
    *   Tracks connected devices via USB/ADB in real time.
    *   Computes library diffs: pushes newly downloaded tracks and deletes files on the device that were removed from the local library.
    *   Structures target directories by artist: `[Music_Folder]/[Artist]/[Track_Name].mp3`.
    *   Cleans up empty artist folders on the remote device when no tracks are left.
    *   Sync locking (`SyncGuard`) prevents concurrent synchronization runs on the same device.
*   **Background Watchdog & Auto-Indexing:** Monitors the downloads folder. Any manual changes (adding/deleting files) trigger an immediate library rescan, memory/DB update, and ADB synchronization.
*   **Proxy Resilience & Fallbacks:** Bypasses regional blocks (SoundCloud geoblocks) by configuring primary and fallback proxies (`fallback_proxies`).
*   **Feature-rich Angular Web Panel:**
    *   **Dashboard:** Displays detailed statistics (total tracks, total size, active downloads), a 7-day activity graph, and a format breakdown chart (MP3, FLAC, WAV).
    *   **SoundCloud Search:** Search tracks, preview audio streaming, and download them with a single click.
    *   **Library Management:**
        *   High-performance virtual scrolling (`Virtual Scrolling`) for smooth rendering of thousands of tracks.
        *   Local fuzzy search (`Fuse.js`) matching titles and artists.
        *   Sorting options by name or date added.
        *   Batch selection and bulk deletion of tracks.
        *   Direct HTTP downloads from the server to your browser.
*   **Built-in Web Audio Player:**
    *   Listen to downloaded tracks or search previews.
    *   System integration via the Media Session API (shows media controls, track title, artist, and artwork in notifications and lock screen).
    *   Global hotkey: spacebar toggles playback (ignores when typing in input fields).
    *   Fisher-Yates shuffle mode and logarithmic volume control, both persisting in local storage.
*   **Embedded redb Database:** Light-weight key-value store database written in Rust. Stores settings, sync states, and cached metadata without external dependencies.

## Quick Start

**1. Via Terminal (Linux):**
```bash
curl -L https://github.com/IVKLD/VortexDL/releases/latest/download/vortex-dl -o vortex-dl && chmod +x vortex-dl
```

**2. Manually:**
1. Go to the [Releases](https://github.com/IVKLD/VortexDL/releases) section.
2. Download the `vortex-dl` binary.
3. Give the file execution permissions: `chmod +x vortex-dl`.
4. (Optional) Move the file to a directory in your `$PATH` (e.g., `/usr/local/bin`) to run it from anywhere.

---
*   Currently, only **Linux** is officially supported.
*   Regarding **macOS** and **Windows**: I'm not a programmer and I have no clue how things work for Mac or Windows users. It should work in theory. If you really need support, open an Issue—I'll try to fix it or wait for contributors.

## Troubleshooting

### "No available download options"
If you see this error:
1.  **99% of the time:** You have a shitty proxy or VPN. SoundCloud simply blocks downloads for your region. It's better to use decent residential IPs.
2.  **The track really can't be downloaded:** This happens sometimes on SC's side, but it's rare. Usually, the issue is point #1.

## Usage

**Download via link:**
```bash
vortex-dl https://soundcloud.com/artist/track
```

**Run Web Interface:**
```bash
vortex-dl --serve
```

**Specify output directory:**
```bash
vortex-dl [URL] --output /path/to/music
```

## Development

### Requirements

For development and building, one of the following is required:

*   **Devbox (Nix):** Recommended method. All dependencies are isolated.
*   **Manual Installation:**
    *   Rust (stable toolchain)
    *   Node.js (18+) and Yarn
    *   [Just](https://github.com/casey/just) (task runner for development and build commands)

### Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/IVKLD/VortexDL.git
   cd VortexDL
   ```
2. Install all dependencies for both backend and frontend:
   ```bash
   just install
   ```
3. Start the components for development:
   ```bash
   # Terminal 1: Backend (runs with hot-reload watch mode)
   just backend

   # Terminal 2: Frontend
   just frontend
   ```
   *The Web Panel (frontend) will be available at http://localhost:4200. The backend REST API is available at http://localhost:3200/api by default.*


For more details on project structure and technical implementation, see [Development.md](Development.md).

## Building

To create a fully optimized production binary with the embedded frontend:
```bash
just dist
```

To build only the backend (without the frontend embedded):
```bash
just build
```

## Futures & Roadmap (What can be implemented)

*   [ ] **YouTube Support:** Downloading audio and video from YouTube tracks, playlists, or channels.
*   [ ] **Native Android Client:** A dedicated lightweight music player app that fetches music from your VortexDL server automatically.
*   [ ] **Global Web App:** A hosted version of the app to download tracks directly in the browser without installing a local server.
*   [ ] **User Account System:** User authentication, personal settings, and custom sync profiles.
*   [ ] **Metadata (ID3 Tag) Editor:** Editing track titles, artists, genre, and artwork directly from the track detail modal.
*   [ ] **Audio Transcoding:** Automatically convert downloaded files to lighter formats (like Opus or AAC) to save storage space on portable devices.
*   [ ] **Wi-Fi Synchronization:** Sync tracks wirelessly using ADB-over-Wi-Fi or a custom local networking protocol.
*   [ ] **Web Panel Customization:** Custom skins, layout options, and light/dark mode switcher.

For more details on future development plans, refer to [ROADMAP.md](ROADMAP.md).

## Manifesto

This project was written because modern streaming services are a piece of shit that won't let you listen to music offline without a fucking subscription. The ability to cache tracks and listen to them offline now costs money, which is just insane.

And those assholes in streaming services remove music at the snap of a finger just because it became "undesirable" to governments. Even though that's more of a gripe with the governments themselves, streaming services bend over for them instantly. In the end, my favorite song can just disappear because some guy in an office decided so.

Another thing that pisses me off is that $1000+ phones no longer have a fucking 3.5mm jack. How am I supposed to connect studio headphones? Shove the jack up my ass and fart out the music? 

Though modern pop trash is no different: "I DROP THE WEST U, I DROP MY ANUS U, I SUCK A DICK U." You give your decent headphones and player to some dipshit, and he says there's no difference. OF COURSE THERE ISN'T, BECAUSE YOU'RE LISTENING TO THE SHIT OF SOME MORON WHO 1) CAN'T MIX 2) CAN'T WRITE 3) CAN'T READ.

Bluetooth and TWS are their own kind of hell with latency and the inability to connect properly to a PC. Why do I have to deal with all this shit if there's a fucking cable that you just plug in AND IT WORKS?

Sure, you can use those USB Type-C to 3.5mm jack adapters, but fuck that—my second adapter just died, and I'm not going to tolerate this technological failure from corporations anymore. Keep eating that shit about how the port is only for charging, even though you can output video from a phone, and that's just a small part of what we're about to lose.

I'm done tolerating it. I decided to show some balls—and here they are. Small, maybe, but at least they're real.

In the end, I bought myself an awesome **Cayin N3 Ultra** player and wrote this crap to download everything from SoundCloud in one click, transfer it via cable, and listen to music like a human being, not a victim of marketing.

## License

This project is distributed under the GNU GPL v3 license.
