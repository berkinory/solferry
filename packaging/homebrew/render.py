#!/usr/bin/env python3
"""Fills the tap's cask and formula from a release's latest.json: render.py LATEST VERSION TAP_DIR"""

import json
import sys
from pathlib import Path

HERE = Path(__file__).parent
FILES = {
    "DMG_AARCH64": "Solferry-macos-aarch64.dmg",
    "DMG_X86_64": "Solferry-macos-x86_64.dmg",
    "CLI_LINUX_AARCH64": "solferry-cli-linux-aarch64.tar.xz",
    "CLI_LINUX_X86_64": "solferry-cli-linux-x86_64.tar.xz",
}


def main():
    latest, version, tap = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
    hashes = json.loads(json.loads(latest.read_text())["manifest"])["files"]
    values = {"VERSION": version, **{key: hashes[name] for key, name in FILES.items()}}
    for template, target in (("solferry.rb.in", "Casks/solferry.rb"), ("solferry-cli.rb.in", "Formula/solferry-cli.rb")):
        text = (HERE / template).read_text()
        for key, value in values.items():
            text = text.replace(f"@{key}@", value)
        (tap / target).parent.mkdir(parents=True, exist_ok=True)
        (tap / target).write_text(text)


if __name__ == "__main__":
    main()
