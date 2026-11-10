#!/usr/bin/env python3
"""Fills the tap's cask and formula from a release's SHA256SUMS: render.py SUMS VERSION TAP_DIR"""

import sys
from pathlib import Path

HERE = Path(__file__).parent
FILES = {
    "DMG_AARCH64": "Solferry-macos-aarch64.dmg",
    "DMG_X86_64": "Solferry-macos-x86_64.dmg",
    "CLI_MACOS_AARCH64": "solferry-cli-macos-aarch64.tar.gz",
    "CLI_MACOS_X86_64": "solferry-cli-macos-x86_64.tar.gz",
    "CLI_LINUX_AARCH64": "solferry-cli-linux-aarch64.tar.gz",
    "CLI_LINUX_X86_64": "solferry-cli-linux-x86_64.tar.gz",
}


def main():
    sums, version, tap = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
    hashes = {line.split()[1]: line.split()[0] for line in sums.read_text().splitlines()}
    values = {"VERSION": version, **{key: hashes[name] for key, name in FILES.items()}}
    for template, target in (("solferry.rb.in", "Casks/solferry.rb"), ("solferry-cli.rb.in", "Formula/solferry-cli.rb")):
        text = (HERE / template).read_text()
        for key, value in values.items():
            text = text.replace(f"@{key}@", value)
        (tap / target).parent.mkdir(parents=True, exist_ok=True)
        (tap / target).write_text(text)


if __name__ == "__main__":
    main()
