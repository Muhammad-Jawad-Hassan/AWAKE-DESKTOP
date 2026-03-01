"""Generates the source app icon and the tray template icon for Awake."""

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"
TRAY_DIR = ROOT / "src-tauri" / "icons"
ASSETS.mkdir(exist_ok=True)
TRAY_DIR.mkdir(parents=True, exist_ok=True)

ACCENT = (255, 214, 0, 255)
BG_TOP = (40, 36, 12, 255)
BG_BOTTOM = (20, 18, 6, 255)

SUPERSAMPLE = 4


def draw_power_glyph(img: Image.Image, cx: float, cy: float, radius: float, stroke: float, fill, bg):
    """A ring with a vertical notch at the top and a bar through the gap - the classic power glyph."""
    draw = ImageDraw.Draw(img)

    draw.ellipse([cx - radius, cy - radius, cx + radius, cy + radius], fill=fill)
    inner = radius - stroke
    draw.ellipse([cx - inner, cy - inner, cx + inner, cy + inner], fill=bg)

    gap_half_width = stroke * 0.95
    gap_top = cy - radius - stroke * 0.4
    gap_bottom = cy - radius * 0.35
    draw.rectangle([cx - gap_half_width, gap_top, cx + gap_half_width, gap_bottom], fill=bg)

    bar_half_width = stroke * 0.42
    bar_top = cy - radius - stroke * 0.55
    bar_bottom = cy - radius * 0.55
    draw.rounded_rectangle(
        [cx - bar_half_width, bar_top, cx + bar_half_width, bar_bottom],
        radius=bar_half_width,
        fill=fill,
    )


def make_app_icon(size: int) -> Image.Image:
    hi = size * SUPERSAMPLE
    img = Image.new("RGBA", (hi, hi), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    pad = hi * 0.06
    radius = hi * 0.225
    for y in range(int(pad), int(hi - pad)):
        t = (y - pad) / (hi - 2 * pad)
        r = round(BG_TOP[0] + (BG_BOTTOM[0] - BG_TOP[0]) * t)
        g = round(BG_TOP[1] + (BG_BOTTOM[1] - BG_TOP[1]) * t)
        b = round(BG_TOP[2] + (BG_BOTTOM[2] - BG_TOP[2]) * t)
        draw.line([(pad, y), (hi - pad, y)], fill=(r, g, b, 255))

    mask = Image.new("L", (hi, hi), 0)
    ImageDraw.Draw(mask).rounded_rectangle([pad, pad, hi - pad, hi - pad], radius=radius, fill=255)
    img.putalpha(mask)

    draw_power_glyph(img, hi * 0.5, hi * 0.52, hi * 0.225, hi * 0.075, ACCENT, BG_BOTTOM)

    return img.resize((size, size), Image.Resampling.LANCZOS)


def make_tray_icon(size: int) -> Image.Image:
    hi = size * SUPERSAMPLE
    img = Image.new("RGBA", (hi, hi), (0, 0, 0, 0))
    black = (0, 0, 0, 255)
    transparent = (0, 0, 0, 0)
    draw_power_glyph(img, hi * 0.5, hi * 0.52, hi * 0.34, hi * 0.115, black, transparent)
    return img.resize((size, size), Image.Resampling.LANCZOS)


def main():
    source = make_app_icon(1024)
    source.save(ASSETS / "icon-source.png")

    tray_2x = make_tray_icon(44)
    tray_2x.save(TRAY_DIR / "tray-icon.png")

    print("Wrote", ASSETS / "icon-source.png")
    print("Wrote", TRAY_DIR / "tray-icon.png")


if __name__ == "__main__":
    main()
