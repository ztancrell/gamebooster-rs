# Hone app (Electron, 2.1.0): notes

Our own words, from reading the bundled JavaScript locally (deleted afterwards). No code is kept here.

## Architecture

- **ow-electron** (Overwolf's Electron fork) app, webpack bundles: `main.js` (Node side), `preload.js`,
  `updater.js` and about 90 renderer chunks (React). The Overwolf app id is in `package.json`.
- The UI is **local** (in the asar), unlike Razer Cortex. Content comes from Hone's REST API under
  `https://hone.gg/api`: the optimization catalogue and order, presets, game presets, announcements,
  banners, changelogs, payments, referrals, game-mode configs and BIOS options. The bundle also
  contains the client for many admin endpoints (user bans, mass premium grants, device-fingerprint
  risk views, benchmark insights). The client is generated from the full API, but the app never calls
  them.
- All system changes go through native or .NET helpers in `resources/assets/hone-resources/`, started
  **elevated**. Elevation goes through `hone-elevation-helper.exe`, a named-pipe broker
  (`\\.\pipe\HoneElevator`) that the app starts with a UAC prompt. The app talks to it over the pipe.
  The broker only runs executables on its allow-list whose Authenticode signature matches a pinned
  certificate, and only writes registry paths on its allow-list (see helpers notes).
- Helpers: `HoneUtility.exe` (one-shot optimizations, `--check` hardware probe, boost-up size
  estimate), `HoneTune.exe` (Game Mode), `GameManager.exe` (game detection and launch via JSON on
  stdin), `SysInfo.exe` (hardware inventory and sensors), `hone-system-validator.exe` (benchmark
  validation), `hone-bios/hone-bios.exe` (BIOS settings), `hone-notifier.exe` (toasts).

## Login and monetization (high level)

- **Login is required**, done in the browser (email, Google or Discord). There is also a "guest" token
  flow. The app stores the user id in `%APPDATA%\hone\user_id.txt`.
- **Free:** 10 optimizations; each referral adds one, up to 15. The app counts applied optimizations
  and shows a threshold warning. **Premium** (subscription, `payments-2` endpoints) removes the limit
  and unlocks premium-only optimizations, "Boost-Ups" (bundles) and scheduled Boost-Ups. It also
  unlocks the Game Mode enhancers CPU affinity, GPU priority, core parking and the game-specific
  CPU0/E-core/SMT switches, and it removes ads. Premium checks are server-side
  (`/optimization/checkPremium/{id}`). Licence enforcement was not studied further.
- The app checks TLS certificates for its own API and refuses ones not issued by a known root. Its
  log message says this is meant to stop MITM "cracks".

## Optimization catalogue

- The catalogue (names, descriptions, categories, premium flags, conflicts, versions) is served by the
  API. Each item has an 8-character id. The renderer bundles **translations for 101–102 items** in 12
  languages (no English table: English comes from the server). The full list, with verdicts, is in
  [docs/hone/README.md](../../../../docs/hone/README.md). Some items may be retired server-side
  (`/optimization/getDeprecated`), so the bundled list may contain items no longer offered.
- To apply an item, the app starts HoneUtility elevated with the item id (several for a Boost-Up).
  It reads progress from stdout and has dedicated exit codes. Before applying, it runs
  `HoneUtility --check`. That returns hardware flags (AMD/Intel CPU, Intel E-cores, AMD multi-CCD,
  NVIDIA GPU, Windows 11 or 10, timer-resolution support, HDD present), which hide items that don't
  fit.
- **Restore:** the app has handlers to enable System Restore, create a restore point and restore it.
  It can revert a game's config optimization ("revertGameOptimization", "revert-pro-player"), and it
  syncs the applied state to the server (`/optimization-sync/sync`). Per-item revert is done by
  HoneUtility's "revert" mode (see its notes). The website says Hone "saves original registry values
  so supported changes can be restored". That could not be verified per item, because HoneUtility's
  code is encrypted.
- **Game configs / "pro player" settings:** game-specific presets that rewrite a game's settings file,
  driven by `/game-preset`, with revert.
- **BIOS:** a renderer table of 87 BIOS option names with "optimal" values (C-states off, C1E off,
  SpeedStep/Speed Shift off, Turbo on, spread spectrum off, Resizable BAR and Above 4G on, fast boot
  off, vendor bloat installers off, Hyper-Threading on, and others). These values are read and written
  by `hone-bios.exe` (see helpers).

## Game Mode in the app

- Game Mode configs ("modes": processes and services lists, GPU priority lists, exclusions) come from
  the API. The app turns them into HoneTune command-line options. Enhancer defaults, from the bundle:
  - Power plan: on (visible). Power throttling, timer resolution and visual effects: **always on**,
    hidden.
  - Premium, off by default: CPU affinity, GPU priority and core parking (general), and disable CPU0,
    disable E-cores (on AMD: second CCD) and disable SMT (game-specific).
- There is a benchmark mode. It runs Hone's own benchmark tools with and without Game Mode and uploads
  the results (`/benchmark`, `/game-mode-sessions`). The results feed the server's per-optimization
  "confidence" insights.

## Telemetry and endpoints

- **Mixpanel** (`api-js.mixpanel.com`) for product analytics, including the Mixpanel **session-replay
  recorder** library. **Sentry** for errors in main and renderer. Hone also has its own
  `/error-service`.
- **Hardware fingerprint:** a "get-hwid" handler, a hardware profile id and a hardware snapshot sync
  (`/user/device-info`). SysInfo inventories CPU, GPU, RAM, disks, Secure Boot, VBS/HVCI and the
  vulnerable-driver blocklist state. The admin API side has "device-fingerprint" risk scoring per
  canonical hardware id (account-sharing and abuse detection).
- Game Mode sessions (game, duration, enhancers, RAM saved, how it ended) and benchmark runs are
  uploaded.
- Overwolf UTM attribution parameters are logged. Uninstall sends a Mixpanel event (see installer).
- Other hosts in the bundle: `help.hone.gg`/`support.hone.gg`, Discord invites, IGDB links, Cloudflare
  Stream video, and the Overwolf update host below.

## Updates

- App updates: electron-updater, "generic" provider at
  `https://electron-updates.overwolf.com/electron-updates/electron/<app id>/`, channel `latest`. The
  update config sets no publisher name, so updates are checked by the SHA-512 in the update manifest
  over HTTPS, not by the updater's Authenticode check. ow-electron may add its own check (not studied).
- Helper updates ("hone-resources"): the app can replace helper files from the API, checked with
  SHA-256 checksums from the server. Files that are in use are replaced through the elevation helper,
  which only launches executables signed with Hone's pinned certificate.
