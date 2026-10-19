"""Render TerminalFlow dashboard snapshots (from `tf screenshot`) to PNG.

Usage: python assets/render_screenshots.py [path/to/dashboard.json]
"""

import json
import os
import sys

from PIL import Image, ImageDraw, ImageFont

CELL_W = 9
CELL_H = 18

NAMED = {
    "Black": "#0d0d0d",
    "Red": "#e06c75",
    "Green": "#98c379",
    "Yellow": "#e5c07b",
    "Blue": "#61afef",
    "Magenta": "#c678dd",
    "Cyan": "#56b6c2",
    "White": "#dcdfe4",
    "Gray": "#5c6370",
    "DarkGray": "#3e4451",
    "LightRed": "#ff7b86",
    "LightGreen": "#b5e890",
    "LightYellow": "#f0d399",
    "LightBlue": "#84c7f4",
    "LightMagenta": "#e3a6f0",
    "LightCyan": "#7fd7e6",
}

DEFAULT_FG = "#dcdfe4"
DEFAULT_BG = "#0d0d0d"

MONO_CANDIDATES = [
    r"C:\Windows\Consola.ttf",
    r"C:\Windows\Fonts\consola.ttf",
    r"C:\Windows\Fonts\lucon.ttf",
    r"C:\Windows\Fonts\cour.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
]
MONO_BOLD_CANDIDATES = [
    r"C:\Windows\Consolab.ttf",
    r"C:\Windows\Fonts\consolab.ttf",
    r"C:\Windows\Fonts\luconbd.ttf",
]
EMOJI_CANDIDATES = [
    r"C:\Windows\Fonts\seguiemj.ttf",
    r"C:\Windows\Fonts\seguiemj2.ttf",
]

NAME_FILE = "terminalflow-dashboard"


def pick(paths):
    for path in paths:
        if os.path.exists(path):
            return path
    return None


def color(value, default):
    if value is None:
        return default
    if isinstance(value, str):
        if value.startswith("#"):
            return value
        return NAMED.get(value, default)
    return default


def is_emoji(symbol):
    return any(ord(ch) > 0x2000 and ch not in "─│┌┐└┘├┤┬┴┼═║╔╗╚╝" for ch in symbol)


def render(snapshot_path, out_dir):
    with open(snapshot_path, "r", encoding="utf-8") as handle:
        data = json.load(handle)

    mono = pick(MONO_CANDIDATES)
    bold = pick(MONO_BOLD_CANDIDATES) or mono
    emoji = pick(EMOJI_CANDIDATES) or mono

    fonts = {
        "regular": ImageFont.truetype(mono, CELL_H - 3),
        "bold": ImageFont.truetype(bold, CELL_H - 3),
        "emoji": ImageFont.truetype(emoji, CELL_H - 3),
    }

    width = data["width"]
    height = data["height"]
    os.makedirs(out_dir, exist_ok=True)

    written = []
    for tab in data["tabs"]:
        name = tab["name"]
        slug = slugify(name)
        image = Image.new("RGB", (width * CELL_W, height * CELL_H), DEFAULT_BG)
        draw = ImageDraw.Draw(image)

        for y, row in enumerate(tab["rows"]):
            for x, cell in enumerate(row):
                symbol = cell.get("s", " ") or " "
                fg = color(cell.get("fg"), DEFAULT_FG)
                bg = color(cell.get("bg"), DEFAULT_BG)
                px = x * CELL_W
                py = y * CELL_H
                if bg != DEFAULT_BG:
                    draw.rectangle([px, py, px + CELL_W, py + CELL_H], fill=bg)
                if symbol.isspace():
                    continue
                if is_emoji(symbol):
                    font = fonts["emoji"]
                elif cell.get("b"):
                    font = fonts["bold"]
                else:
                    font = fonts["regular"]
                draw.text((px + 1, py + 1), symbol, font=font, fill=fg)

        out_path = os.path.join(out_dir, f"{slug}.png")
        image.save(out_path, optimize=True)
        written.append(out_path)
        print(f"wrote {out_path} ({image.width}x{image.height})")

    return written


def slugify(name):
    keep = [ch if (ch.isalnum() or ch == "-") else "-" for ch in name.lower()]
    slug = "".join(keep)
    while "--" in slug:
        slug = slug.replace("--", "-")
    return slug.strip("-")


if __name__ == "__main__":
    snapshot = sys.argv[1] if len(sys.argv) > 1 else "assets/dashboard.json"
    out_dir = sys.argv[2] if len(sys.argv) > 2 else "assets"
    if not os.path.exists(snapshot):
        sys.exit(f"snapshot not found: {snapshot} (run: tf screenshot --out {snapshot})")
    render(snapshot, out_dir)
