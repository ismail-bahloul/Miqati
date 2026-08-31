"""Generate the README banner for Miqati.

Dark Fluent surface with the accent-blue crescent, the product name and the
real widget UI (compact + detail screenshots) on an elevated card.
Output: assets/banner.png (1280x400).
"""
import math
from PIL import Image, ImageDraw, ImageFont

W, H = 1280, 400
ACCENT = (96, 205, 255)      # #60cdff
ACCENT_DIM = (60, 160, 210)
BG_TOP = (46, 46, 46)        # #2E2E2E
BG_BOTTOM = (28, 28, 28)     # #1C1C1C
TEXT = (255, 255, 255)
TEXT_DIM = (154, 154, 154)

FONT = "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf"
FONT_REG = "/usr/share/fonts/TTF/DejaVuSans.ttf"


def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def crescent(d, cx, cy, ro, ri, off, color):
    d.ellipse([cx - ro, cy - ro, cx + ro, cy + ro], fill=color)
    d.ellipse([cx - ri + off, cy - ri - off, cx + ri + off, cy + ri - off], fill=(0, 0, 0, 0))


def star(d, cx, cy, r, color):
    pts = []
    for i in range(5):
        for s in (r, r * 0.4):
            a = math.radians(i * 72 + (0 if s == r else 36) - 90)
            pts.append((cx + s * math.cos(a), cy + s * math.sin(a)))
    d.polygon(pts, fill=color)


# Gradient background.
img = Image.new("RGBA", (W, H))
d = ImageDraw.Draw(img)
for y in range(H):
    d.line([(0, y), (W, y)], fill=lerp(BG_TOP, BG_BOTTOM, y / H) + (255,))

# Brand mark (crescent + star) on the left.
crescent(d, 168, 200, 95, 80, 27, ACCENT + (255,))
star(d, 246, 150, 22, ACCENT + (255,))

# Product name + tagline (center-left).
f_title = ImageFont.truetype(FONT, 84)
f_tag = ImageFont.truetype(FONT_REG, 28)
title = "Miqati"
tw = d.textlength(title, font=f_title)
d.text((640 - tw / 2, 110), title, font=f_title, fill=TEXT + (255,))
tag = "your prayer times"
tgw = d.textlength(tag, font=f_tag)
d.text((640 - tgw / 2, 208), tag, font=f_tag, fill=TEXT_DIM + (255,))

# Subtle accent line under the tagline.
d.line([(470, 250), (810, 250)], fill=ACCENT_DIM + (60,), width=2)

# Right: real widget UI on an elevated rounded card.
detail = Image.open("assets/screenshot-detail.png").convert("RGBA").resize((168, 204))
compact = Image.open("assets/screenshot-compact.png").convert("RGBA").resize((168, 42))
cx0, cy0, cw, ch = 980, 78, 210, 300
d.rounded_rectangle([cx0, cy0, cx0 + cw, cy0 + ch], radius=18,
                    fill=(20, 20, 20, 130), outline=(255, 255, 255, 22), width=1)
img.paste(detail, (cx0 + 21, cy0 + 16), detail)
img.paste(compact, (cx0 + 21, cy0 + ch - 58), compact)

img.save("assets/banner.png")
print("saved assets/banner.png", img.size)
