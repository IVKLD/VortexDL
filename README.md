<p align="center">
  <img src="assets/logo.svg" alt="VortexDL Logo" width="128">
</p>

<h3 align="center">VortexDL</h3>

<p align="center">
  Multi-threaded music downloader & synchronization station for SoundCloud and YouTube with a sleek Angular web panel.
  <br>
  <a href="https://github.com/IVKLD/VortexDL/releases">
    <img src="https://img.shields.io/github/v/release/IVKLD/VortexDL?style=flat-square" alt="Release">
  </a>
  <img src="https://img.shields.io/badge/License-GPLv3-blue.svg?style=flat-square" alt="License">
  <img src="https://img.shields.io/badge/Rust-2024_Edition-orange?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/Angular-v22-DD0031?style=flat-square&logo=angular&logoColor=white" alt="Angular">
</p>

## Motivation
Streaming services are a piece of shit with subscriptions and censorship, and modern phones without a 3.5mm jack are a technological failure. This project was written to download music in one click and listen to it like a human being on a proper player (e.g., **Cayin N3 Ultra**).

Read the full thoughts on this — [at the bottom of the page](#manifesto).

## How it works

*   **High-Performance Rust Backend:** Uses native asynchronous clients (`soundcloud-rs` and `yt-audio-downloader`) for unthrottled concurrent multi-threaded track downloading, stream resolution, and automatic ID3 metadata tagging.
*   **Dual Platform Discovery:** Search, stream previews, and download directly from both **SoundCloud** and **YouTube** (including playlists and direct URLs).
*   **Real-time Progress:** Live progress tracking across CLI terminal bars and reactive WebSocket / HTTP server-sent events in the web UI.
*   **Angular Web Panel:** Sleek, modern web panel with virtual scrolling, fuzzy search, statistics dashboard, and embedded web audio player.
*   **Automatic ADB Device Synchronization:** Automatically detects USB-connected Android devices, computes library differences, synchronizes audio files into organized artist directories, and cleans orphaned folders.
*   **Single Self-Contained Binary:** Complete web application and backend bundled into a single standalone executable.

## Features

*   **Multi-Platform High-Speed Downloading:**
    *   **SoundCloud**: Concurrent progressive MP3 and HLS audio stream fetching.
    *   **YouTube**: Unthrottled parallel chunked byte-range downloads bypassing YouTube CDN rate limits, with automatic Opus/AAC to MP3/FLAC/WAV conversion.
*   **ADB Synchronization for Android Devices:**
    *   Tracks connected devices via USB/ADB in real time.
    *   Computes library diffs: pushes newly downloaded tracks and deletes files on the device that were removed from the local library.
    *   Structures target directories by artist: `[Music_Folder]/[Artist]/[Track_Name].mp3`.
    *   Cleans up empty artist folders on the remote device when no tracks are left.
    *   Sync locking (`SyncGuard`) prevents concurrent synchronization runs on the same device.
*   **Background Watchdog & Auto-Indexing:** Monitors the downloads folder in real time. Manual additions or deletions instantly update the in-memory cache, database, and trigger automatic ADB sync.
*   **Proxy Resilience & Fallbacks:** Bypasses regional geoblocks with primary and fallback racing proxy pools.
*   **Feature-rich Angular Web Panel:**
    *   **Dashboard:** Displays detailed statistics (total tracks, library size, active download queue), 7-day activity graphs, and format breakdown charts.
    *   **Multi-Provider Search:** Unified search toolbar with inverted curved tabs for fast toggling between YouTube and SoundCloud.
    *   **Library Management:**
        *   High-performance virtual scrolling (`@angular/cdk/scrolling`) for silky-smooth browsing of thousands of tracks.
        *   Local fuzzy search (`Fuse.js`) matching titles and artists.
        *   Sorting by title, artist, or date added.
        *   Batch selection and bulk deletion.
        *   Direct HTTP downloads from server to browser.
*   **Built-in Web Audio Player:**
    *   Listen to downloaded tracks or live search previews.
    *   System integration via Media Session API (lock screen controls, title, artist, artwork).
    *   Global keyboard shortcuts (spacebar toggle).
    *   Fisher-Yates shuffle mode and logarithmic volume persistence.
*   **Embedded redb Database:** Ultra-fast, zero-dependency key-value store for configuration, sync state, and cached metadata.
*   **Backup & Cloud Sync:** Export and import library snapshots across Local Storage, URL endpoints, and WebDAV servers.

## Quick Start

**1. Via Terminal (Linux):**
```bash
curl -L https://github.com/IVKLD/VortexDL/releases/latest/download/vortex-dl -o vortex-dl && chmod +x vortex-dl
```

**2. Manually:**
1. Go to the [Releases](https://github.com/IVKLD/VortexDL/releases) section.
2. Download the `vortex-dl` binary.
3. Make it executable: `chmod +x vortex-dl`.
4. (Optional) Move to your `$PATH` (e.g., `/usr/local/bin`).

---
*   Currently, **Linux** is officially supported.

### NixOS & Nix Flakes Setup

VortexDL includes a complete `flake.nix` with dev shells, package derivations, and NixOS / Home Manager modules:

#### 1. Run directly via Flakes
```bash
nix run --refresh github:IVKLD/VortexDL -- --serve
```

#### 2. Development Shell
```bash
nix develop
```

#### 3. Install binary to Nix profile
```bash
nix profile install github:IVKLD/VortexDL
```

#### 4. NixOS System Configuration Example (`flake.nix`)

```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    vortexdl.url = "github:IVKLD/VortexDL";
  };

  outputs = { self, nixpkgs, vortexdl, ... }: {
    nixosConfigurations.myhostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        vortexdl.nixosModules.vortexdl
        ({ config, pkgs, ... }: {
          services.vortexdl = {
            enable = true;
            port = 3200;
            openFirewall = true;
            downloadDir = "/home/user/Music";
          };
        })
      ];
    };
  };
}
```

## Usage

**Download via URL (SoundCloud or YouTube):**
```bash
vortex-dl https://soundcloud.com/artist/track
vortex-dl https://www.youtube.com/watch?v=dQw4w9WgXcQ
```

**Launch Web Interface:**
```bash
vortex-dl --serve
```

**Specify Output Directory:**
```bash
vortex-dl [URL] --output /path/to/music
```

## Development

### Requirements

*   **Rust** (2024 edition, stable)
*   **Node.js** (18+) and **Yarn**
*   **[Just](https://github.com/casey/just)** (task runner)
*   **FFmpeg** (for audio transcoding)

### Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/IVKLD/VortexDL.git
   cd VortexDL
   ```
2. Install dependencies:
   ```bash
   just install
   ```
3. Run development servers:
   ```bash
   # Terminal 1: Backend (with auto-reload watch mode)
   just backend

   # Terminal 2: Frontend
   just frontend
   ```
   *Web Panel: `http://localhost:4200` | REST API: `http://localhost:3200/api`*

### Building

To build the self-contained production binary with the embedded Angular frontend:
```bash
just dist
```

## Manifesto

This project was written because modern streaming services are a piece of shit that won't let you listen to music offline without a fucking subscription. The ability to cache tracks and listen to them offline now costs money, which is just insane.

And those assholes in streaming services remove music at the snap of a finger just because it became "undesirable" to governments. Even though that's more of a gripe with the governments themselves, streaming services bend over for them instantly. In the end, my favorite song can just disappear because some guy in an office decided so.

Another thing that pisses me off is that $1000+ phones no longer have a fucking 3.5mm jack. How am I supposed to connect studio headphones? Shove the jack up my ass and fart out the music? 

Though modern pop trash is no different: "I DROP THE WEST U, I DROP MY ANUS U, I SUCK A DICK U." You give your decent headphones and player to some dipshit, and he says there's no difference. OF COURSE THERE ISN'T, BECAUSE YOU'RE LISTENING TO THE SHIT OF SOME MORON WHO 1) CAN'T MIX 2) CAN'T WRITE 3) CAN'T READ.

Bluetooth and TWS are their own kind of hell with latency and the inability to connect properly to a PC. Why do I have to deal with all this shit if there's a fucking cable that you just plug in AND IT WORKS?

Sure, you can use those USB Type-C to 3.5mm jack adapters, but fuck that—my second adapter just died, and I'm not going to tolerate this technological failure from corporations anymore. Keep eating that shit about how the port is only for charging, even though you can output video from a phone, and that's just a small part of what we're about to lose.

I'm done tolerating it. I decided to show some balls—and here they are. Small, maybe, but at least they're real.

In the end, I bought myself an awesome **Cayin N3 Ultra** player and wrote this crap to download everything from SoundCloud and YouTube in one click, automatically sync it to the device over ADB when plugged in, and listen to music like a human being, not a marketing victim.

## License

GNU General Public License v3.0 (`GPL-3.0-only`).
