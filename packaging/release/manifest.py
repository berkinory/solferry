#!/usr/bin/env python3
"""Writes the update manifest for everything in a release folder: manifest.py DIST VERSION OUT

The manifest lists every file's hash, and the apps install the files under UPDATES. The workflow
signs these exact bytes and wraps them with the signature into latest.json."""

import hashlib
import json
import sys
from pathlib import Path

REPOSITORY = "https://github.com/berkinory/solferry"
UPDATES = {
    "macos-aarch64": "Solferry-macos.dmg",
    "macos-x86_64": "Solferry-macos.dmg",
    "windows-x86_64": "Solferry-windows-setup.exe",
    "linux-x86_64": "Solferry-linux-x86_64.AppImage",
    "linux-aarch64": "Solferry-linux-aarch64.AppImage",
}


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as file:
        while chunk := file.read(1 << 20):
            digest.update(chunk)
    return digest.hexdigest()


def main():
    dist, version, out = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
    files = {path.name: sha256(path) for path in sorted(dist.iterdir()) if path.is_file()}
    missing = sorted({name for name in UPDATES.values() if name not in files})
    if missing:
        raise SystemExit(f"missing update files: {missing}")
    assets = {
        platform: {
            "url": f"{REPOSITORY}/releases/download/v{version}/{name}",
            "sha256": files[name],
            "size": (dist / name).stat().st_size,
        }
        for platform, name in UPDATES.items()
    }
    page = f"{REPOSITORY}/releases/tag/v{version}"
    manifest = {"version": version, "page": page, "assets": assets, "files": files}
    out.write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
