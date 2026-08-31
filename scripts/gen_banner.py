"""Generate the README banner for Miqati.

Dark Fluent surface with the accent-blue crescent + star and the product name.
Output: assets/banner.png (1280x400).
"""
import math
from PIL import Image, ImageDraw, ImageFont

W, H = 1280, 400
ACCENT = (96, 205, 255)      # #60cdff
ACCENT_DIM = (60, 160, 210)  # darker accent for the ring
BG_TOP = (46, 46, 46)        # #2E2E2E
BG_BOTTOM = (28, 28, 28)     # #1C1C1C
TEXT = (255, 255, 255)
TEXT_DIM = (154, 154, 154)

FONT = "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf"
FONT_REG = "/usr/share/fonts/TTF/DejaVuSans.ttf"


def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def draw_crescent(d, cx, cy, r_outer, r_inner, offset, color):
    d.ellipse([cx - r_outer, cy - r_outer, cx + r_outer, cy + r_outer], fill=color)
    d.ellipse(
        [cx - r_inner + offset, cy - r_inner - offset,
         cx + r_inner + offset, cy + r_inner - offset],
        fill=(0, 0, 0, 0),
    )


def draw_star(d, cx, cy, r, color):
    pts = []
    for i in range(5):
        for scale in (r, r * 0.4):
            a = math.radians(i * 72 + (0 if scale == r else 36) - 90)
            pts.append((cx + scale * math.cos(a), cy + scale * math.sin(a)))
    d.polygon(pts, fill=color)


# Gradient background.
img = Image.new("RGBA", (W, H))
d = ImageDraw.Draw(img)
for y in range(H):
    d.line([(0, y), (W, y)], fill=lerp(BG_TOP, BG_BOTTOM, y / H) + (255,))

# Crescent + star on the left.
draw_crescent(d, 230, H // 2, 105, 89, 30, ACCENT + (255,))
draw_star(d, 318, 150, 24, ACCENT + (255,))

# Subtle accent ring behind the text area (definition).
d.line([(60, H - 46), (W - 60, H - 46)], fill=ACCENT_DIM + (70,), width=2)

# Product name.
title = "Miqati"
try:
    font_title = ImageFont.truetype(FONT, 96)
except OSError:
    font_title = ImageFont.load_default()
w = d.textlength(title, font=font_title)
d.text(((W - w) / 2 + 40, 132), title, font=font_title, fill=TEXT + (255,))

# Tagline.
try:
    font_tag = ImageFont.truetype(FONT_REG, 30)
except OSError:
    font_tag = ImageFont.load_default()
tag = "vos horaires de prière"
wt = d.textlength(tag, font=font_tag)
d.text(((W - wt) / 2 + 40, 250), tag, font=font_tag, fill=TEXT_DIM + (255,))

img.save("assets/banner.png")
print("saved assets/banner.png", img.size)
