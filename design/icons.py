#!/usr/bin/env python3
"""Writes the app's icons as SVG files for the desktop, vector drawables for Android and image sets
for iOS: interface
icons from @hugeicons/core-free-icons, platform logos from Simple Icons.

Usage: design/icons.py <path to the core-free-icons package>
"""

import json
import re
import sys
import urllib.request
import xml.etree.ElementTree as ElementTree
from pathlib import Path

OUT = Path(__file__).parent.parent / "crates/solferry-desktop/assets/icons"
ANDROID = Path(__file__).parent.parent / "android/app/src/main/res/drawable"
IOS = Path(__file__).parent.parent / "ios/Solferry/Assets.xcassets"
MARK = OUT.parent / "mark.svg"
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
    "history": "Clock01Icon",
    "home": "Home01Icon",
    "key": "Key01Icon",
    "link": "Link01Icon",
    "more": "MoreHorizontalIcon",
    "nearby": "HotspotIcon",
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


def circle_path(element):
    cx, cy, r = (float(element.get(name)) for name in ("cx", "cy", "r"))
    return f"M{cx - r},{cy}a{r},{r} 0 1,0 {2 * r},0a{r},{r} 0 1,0 {-2 * r},0"


def rect_path(element):
    x, y, width, height = (float(element.get(name, 0)) for name in ("x", "y", "width", "height"))
    r = float(element.get("rx", 0))
    across, down = width - 2 * r, height - 2 * r
    return (
        f"M{x + r},{y}h{across}a{r},{r} 0 0 1 {r},{r}v{down}a{r},{r} 0 0 1 {-r},{r}"
        f"h{-across}a{r},{r} 0 0 1 {-r},{-r}v{-down}a{r},{r} 0 0 1 {r},{-r}z"
    )


def android_vector(svg):
    """The same icon as an Android vector drawable, tinted by whoever draws it."""
    root = ElementTree.fromstring(svg)
    size = root.get("viewBox", "0 0 24 24").split()[2]
    filled_by_default = root.get("fill") != "none"
    paths = []
    for element in root.iter():
        tag = element.tag.rsplit("}", 1)[-1]
        if tag not in ("path", "circle", "rect"):
            continue
        data = {"path": lambda e: e.get("d"), "circle": circle_path, "rect": rect_path}[tag](element)
        stroke = element.get("stroke")
        fill = element.get("fill", "currentColor" if filled_by_default else "none")
        parts = [f'android:pathData="{data}"']
        if fill != "none":
            parts.append('android:fillColor="#FF000000"')
        if stroke and stroke != "none":
            parts.append('android:strokeColor="#FF000000"')
            parts.append(f'android:strokeWidth="{element.get("stroke-width", "1")}"')
            caps = {"round": "round", "square": "square"}
            if element.get("stroke-linecap") in caps:
                parts.append(f'android:strokeLineCap="{element.get("stroke-linecap")}"')
            if element.get("stroke-linejoin") in ("round", "bevel"):
                parts.append(f'android:strokeLineJoin="{element.get("stroke-linejoin")}"')
        paths.append("    <path " + " ".join(parts) + " />")
    head = (
        '<vector xmlns:android="http://schemas.android.com/apk/res/android"\n'
        f'    android:width="24dp" android:height="24dp"\n'
        f'    android:viewportWidth="{size}" android:viewportHeight="{size}">\n'
    )
    return head + "\n".join(paths) + "\n</vector>\n"


def write_android():
    ANDROID.mkdir(parents=True, exist_ok=True)
    for svg in sorted(OUT.glob("*.svg")):
        name = "ic_" + svg.stem.replace("-", "_") + ".xml"
        (ANDROID / name).write_text(android_vector(svg.read_text()))


def expanded_arcs(svg):
    """iOS can't read arcs whose flags are packed against the numbers after them."""
    number = re.compile(r"\s*,?\s*([-+]?(?:\d*\.\d+|\d+\.?)(?:[eE][-+]?\d+)?)")
    flag = re.compile(r"\s*,?\s*([01])")

    def spaced(found):
        arcs, at = [], 0
        arguments = found.group(2)
        while arguments[at:].strip(" ,"):
            values = []
            for pattern in (number, number, number, flag, flag, number, number):
                match = pattern.match(arguments, at)
                values.append(match.group(1))
                at = match.end()
            arcs.append(" ".join(values))
        return found.group(1) + " ".join(arcs) + " "

    return re.sub(r"([aA])([^a-zA-Z]+)", spaced, svg)


def write_ios():
    about = {"info": {"author": "xcode", "version": 1}}
    for svg in [*sorted(OUT.glob("*.svg")), MARK]:
        folder = IOS / f"{svg.stem}.imageset"
        folder.mkdir(parents=True, exist_ok=True)
        (folder / svg.name).write_text(expanded_arcs(svg.read_text()).replace("currentColor", "#000000"))
        contents = {
            "images": [{"filename": svg.name, "idiom": "universal"}],
            **about,
            "properties": {"preserves-vector-representation": True, "template-rendering-intent": "template"},
        }
        (folder / "Contents.json").write_text(json.dumps(contents, indent=2) + "\n")


def main():
    package = Path(sys.argv[1]) / "dist/esm"
    OUT.mkdir(parents=True, exist_ok=True)
    for name, icon in ICONS.items():
        (OUT / f"{name}.svg").write_text(convert((package / f"{icon}.js").read_text()))
    for name in BRANDS:
        (OUT / f"{name}.svg").write_text(brand(name))
    (OUT / "windows.svg").write_text(WINDOWS)
    write_android()
    (ANDROID / "ic_mark.xml").write_text(android_vector(MARK.read_text()))
    write_ios()


if __name__ == "__main__":
    main()
