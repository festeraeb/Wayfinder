#!/usr/bin/env python3
"""Publish a signed Wayfinder update (run on the machine that holds the signing key).

  publish_update.py --installer Wayfinder_2.1.1_x64-setup.exe --version 2.1.1 [--notes "text"] [--dry-run]

What it does:
  1. signs the installer with the Tauri updater key (private key + password files stay where they are,
     mode 600; neither is read into this script's output),
  2. copies the installer and its .sig under <web root>/<version>/,
  3. writes latest.json atomically (the file the app checks): version, notes, pub_date and the
     windows-x86_64 entry with the signature and the download URL.

Defaults: key dir ~/.config/wayfinder-updater/ (updater.key, key.password), web root
/data/www/cesarops/wayfinder/updates, public base https://cesarops.com/wayfinder/updates.
The update check is a plain GET of latest.json: no version, platform or id is sent.
"""
import argparse
import datetime
import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

HOME = pathlib.Path.home()
KEY_DIR = HOME / ".config" / "wayfinder-updater"
WEB_ROOT = pathlib.Path("/data/www/cesarops/wayfinder/updates")
BASE_URL = "https://cesarops.com/wayfinder/updates"
SEMVER = re.compile(r"^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$")


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def build_manifest(version, notes, signature, url, platform="windows-x86_64", now=None):
    """The Tauri updater's static manifest format."""
    if not SEMVER.match(version):
        raise ValueError(f"not a semantic version: {version!r}")
    now = now or datetime.datetime.now(datetime.timezone.utc)
    return {
        "version": version,
        "notes": notes,
        "pub_date": now.strftime("%Y-%m-%dT%H:%M:%SZ"),
        "platforms": {platform: {"signature": signature, "url": url}},
    }


def write_atomic(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(dir=path.parent, prefix=".latest-")
    with os.fdopen(fd, "w") as f:
        f.write(text)
    os.chmod(tmp, 0o644)
    os.replace(tmp, path)


def sign(installer, key_dir):
    """Sign with the Tauri CLI; returns the signature text. Nothing from the key files is printed."""
    key, pw = key_dir / "updater.key", key_dir / "key.password"
    for p in (key, pw):
        if not p.exists():
            sys.exit(f"missing {p}")
        if p.stat().st_mode & 0o077:
            sys.exit(f"{p} must be mode 600")
    env = dict(os.environ, TAURI_SIGNING_PRIVATE_KEY_PATH=str(key), TAURI_SIGNING_PRIVATE_KEY_PASSWORD=pw.read_text().strip())
    repo = pathlib.Path(__file__).resolve().parents[1]
    r = subprocess.run(["npx", "--no-install", "tauri", "signer", "sign", str(installer)], cwd=repo, env=env, capture_output=True, text=True)
    sig = pathlib.Path(str(installer) + ".sig")
    if r.returncode != 0 or not sig.exists():
        sys.exit("signing failed: " + (r.stderr.strip().splitlines() or ["unknown error"])[-1])
    return sig


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--installer", required=True, type=pathlib.Path, help="NSIS setup .exe built for this version")
    ap.add_argument("--version", required=True)
    ap.add_argument("--notes", default="")
    ap.add_argument("--platform", default="windows-x86_64")
    ap.add_argument("--key-dir", type=pathlib.Path, default=KEY_DIR)
    ap.add_argument("--web-root", type=pathlib.Path, default=WEB_ROOT)
    ap.add_argument("--base-url", default=BASE_URL)
    ap.add_argument("--dry-run", action="store_true", help="sign and print the manifest, publish nothing")
    a = ap.parse_args(argv)

    if not a.installer.is_file():
        sys.exit(f"no such installer: {a.installer}")
    if not SEMVER.match(a.version):
        sys.exit(f"not a semantic version: {a.version}")
    work = pathlib.Path(tempfile.mkdtemp(prefix="wf-publish-"))
    try:
        name = f"Wayfinder_{a.version}_x64-setup.exe"
        staged = work / name
        shutil.copy2(a.installer, staged)
        sig_file = sign(staged, a.key_dir)
        signature = sig_file.read_text().strip()
        url = f"{a.base_url.rstrip('/')}/{a.version}/{name}"
        manifest = build_manifest(a.version, a.notes, signature, url, a.platform)
        digest = sha256(staged)
        if a.dry_run:
            print(json.dumps(manifest, indent=2))
            print("sha256", digest)
            return 0
        dest = a.web_root / a.version
        dest.mkdir(parents=True, exist_ok=True)
        shutil.copy2(staged, dest / name)
        shutil.copy2(sig_file, dest / (name + ".sig"))
        # installer first, manifest last: a client never sees a manifest whose file is missing
        write_atomic(a.web_root / "latest.json", json.dumps(manifest, indent=2) + "\n")
        print(f"published {a.version}: {url}")
        print(f"sha256 {digest}")
        return 0
    finally:
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
