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

## Requirements

For development and building, one of the following is required:

*   **Devbox (Nix):** Recommended method. All dependencies are isolated.
*   **Manual Installation:**
    *   Rust (stable toolchain)
    *   Node.js (18+) and Yarn

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

1. Clone the repository:
   ```bash
   git clone https://github.com/IVKLD/VortexDL.git
   cd VortexDL
   ```
2. Run components separately (recommended for development):
   ```bash
   # Terminal 1: Backend
   cargo run -- --serve

   # Terminal 2: Frontend
   cd frontend && yarn start
   ```
   *The Web Panel (frontend) will be available at http://localhost:4200. The backend REST API is available at http://localhost:3200/api by default (can be changed via the VORTEX_PORT environment variable).*

*Note: The frontend is automatically built and embedded into the binary during backend compilation.*


For more details on project structure and technical implementation, see [Development.md](Development.md).

## Building

To create an optimized binary:
```bash
cargo build --release
```

## Roadmap
Future plans can be found in [ROADMAP.md](ROADMAP.md).

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
