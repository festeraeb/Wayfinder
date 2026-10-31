# Updates

Wayfinder updates itself from signed releases using Tauri's updater plugin.

- **Channel:** `https://cesarops.com/wayfinder/updates/latest.json`, a static file. The check is a plain GET:
  it sends no version, platform, id or text. The app compares versions itself and verifies the installer's
  signature against the public key in `src-tauri/tauri.conf.json` before installing.
- **When:** shortly after start and every `check_interval_minutes` (default 360).
- **Settings:** `%APPDATA%\wayfinder\updater_config.json` (`~/.config/wayfinder/` elsewhere), created on first run:
  `update_token`, `check_interval_minutes`, `auto_install` (default false: a background check never restarts you),
  `push_port` (14322), `bind_tailnet` (true), `enabled`.
- **Push endpoint** (loopback and the tailnet only; never `0.0.0.0`), `Authorization: Bearer <update_token>`:
  - `POST /api/update/check-now` checks and, if a newer version exists, installs it and restarts. Add `?install=0` to check only.
  - `GET /api/update/status` reports the running version and the last check.

## Publishing (maintainer, on the machine with the signing key)

1. Build the NSIS installer for the new version: `npm run tauri build -- --bundles nsis`.
2. `python3 scripts/publish_update.py --installer <setup.exe> --version X.Y.Z --notes "..."`
   signs it with the key in `~/.config/wayfinder-updater/` (never committed), copies it to the update directory
   and writes `latest.json` last.
3. Optionally push to the fleet laptops: `cesaropsserver/scripts/wayfinder_push_update.py`.

The public key is in the repo; the private key and its password are not, and must never be.
