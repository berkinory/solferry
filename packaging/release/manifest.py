#!/usr/bin/env python3
"""Writes SHA256SUMS and the update manifest for everything in the given folder."""

import hashlib
import json
import sys
from pathlib import Path

REPOSITORY = "https://github.com/berkinory/solferry"
UPDATES = {
    "macos-aarch64": "Solferry-macos-aarch64.zip",
    "macos-x86_64": "Solferry-macos-x86_64.zip",
    "windows-x86_64": "Solferry-windows-x86_64.exe",
    "windows-aarch64": "Solferry-windows-aarch64.exe",
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
    dist, version = Path(sys.argv[1]), sys.argv[2]
    hashes = {path.name: sha256(path) for path in sorted(dist.iterdir()) if path.is_file()}
    (dist / "SHA256SUMS").write_text("".join(f"{value}  {name}\n" for name, value in hashes.items()))
    missing = [name for name in UPDATES.values() if name not in hashes]
    if missing:
        raise SystemExit(f"missing update files: {missing}")
    assets = {
        platform: {
            "url": f"{REPOSITORY}/releases/download/v{version}/{name}",
            "sha256": hashes[name],
            "size": (dist / name).stat().st_size,
        }
        for platform, name in UPDATES.items()
    }
    page = f"{REPOSITORY}/releases/tag/v{version}"
    manifest = {"version": version, "page": page, "assets": assets}
    (dist / "latest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
