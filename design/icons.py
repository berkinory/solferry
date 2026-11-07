#!/usr/bin/env python3
"""Writes the app's icons as SVG files: interface icons from @hugeicons/core-free-icons,
platform logos from Simple Icons.

Usage: design/icons.py <path to the core-free-icons package>
"""

import re
import sys
import urllib.request
from pathlib import Path

OUT = Path(__file__).parent.parent / "crates/solferry-desktop/assets/icons"
ICONS = {
    "add": "Add01Icon",
    "alert": "Alert02Icon",
    "arrow-left": "ArrowLeft01Icon",
    "blocked": "UnavailableIcon",
    "cancel": "Cancel01Icon",
    "check": "Tick02Icon",
    "clipboard": "ClipboardIcon",
    "computer": "ComputerIcon",
    "copy": "Copy01Icon",
    "delete": "Delete02Icon",
    "devices": "LaptopPhoneSyncIcon",
    "download": "Download04Icon",
    "external": "LinkSquare02Icon",
    "file": "File01Icon",
    "folder": "Folder01Icon",
    "folder-open": "FolderOpenIcon",
    "key": "Key01Icon",
    "more": "MoreHorizontalIcon",
    "nearby": "Wifi01Icon",
    "qr": "QrCodeIcon",
    "relay": "ServerStack01Icon",
    "send": "ArrowUp02Icon",
    "settings": "Settings02Icon",
    "shield": "Shield01Icon",
    "transfers": "ArrowDataTransferHorizontalIcon",
    "upload": "Upload04Icon",
    "view": "ViewIcon",
}
SIMPLE_ICONS = "https://cdn.jsdelivr.net/npm/simple-icons@13.21.0/icons/{}.svg"
BRANDS = ("apple", "android", "linux")
# Simple Icons dropped Microsoft's logos at Microsoft's request, so this one is drawn here.
WINDOWS = (
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">'
    '<path d="M2 2h9.5v9.5H2zM12.5 2H22v9.5h-9.5zM2 12.5h9.5V22H2zM12.5 12.5H22V22h-9.5z"/></svg>\n'
)
ELEMENT = re.compile(r'\["(\w+)",\s*\{(.*?)\}\]', re.S)
ATTRIBUTE = re.compile(r'(\w+):\s*"([^"]*)"')


def kebab(name):
    return re.sub(r"([A-Z])", lambda match: "-" + match.group(1).lower(), name)


def convert(source):
    parts = []
    for tag, body in ELEMENT.findall(source):
        attributes = " ".join(
            f'{kebab(key)}="{value}"' for key, value in ATTRIBUTE.findall(body) if key != "key"
        )
        parts.append(f"<{tag} {attributes}/>")
    head = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none">'
    return head + "".join(parts) + "</svg>\n"


def brand(name):
    with urllib.request.urlopen(SIMPLE_ICONS.format(name), timeout=30) as response:
        svg = response.read().decode()
    return re.sub(r'<title>.*?</title>| role="img"', "", svg).strip() + "\n"


def main():
    package = Path(sys.argv[1]) / "dist/esm"
    OUT.mkdir(parents=True, exist_ok=True)
    for name, icon in ICONS.items():
        (OUT / f"{name}.svg").write_text(convert((package / f"{icon}.js").read_text()))
    for name in BRANDS:
        (OUT / f"{name}.svg").write_text(brand(name))
    (OUT / "windows.svg").write_text(WINDOWS)


if __name__ == "__main__":
    main()
