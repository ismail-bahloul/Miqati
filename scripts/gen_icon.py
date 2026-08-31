"""Generate the app icon for Miqati.

Windows 11 dark Fluent surface (#2E2E2E -> #1C1C1C) with an accent-blue
(#60cdff) crescent + star, matching the widget's visual identity.
"""
import math
from PIL import Image, ImageDraw

SIZE = 1024
ACCENT = (96, 205, 255)     # #60cdff
BG_TOP = (46, 46, 46)       # #2E2E2E
BG_BOTTOM = (28, 28, 28)    # #1C1C1C
RADIUS = 180
M = 20


def lerp(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


# Rounded mask.
mask = Image.new("L", (SIZE, SIZE), 0)
ImageDraw.Draw(mask).rounded_rectangle([M, M, SIZE - M, SIZE - M], radius=RADIUS, fill=255)

# Vertical gradient background (Fluent dark).
grad = Image.new("RGBA", (SIZE, SIZE))
gd = ImageDraw.Draw(grad)
for y in range(SIZE):
    gd.line([(0, y), (SIZE, y)], fill=lerp(BG_TOP, BG_BOTTOM, y / SIZE) + (255,))

img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
img.paste(grad, (0, 0), mask)
draw = ImageDraw.Draw(img)

# Crescent = filled circle minus an offset circle.
cx = cy = SIZE // 2
r_outer, r_inner, offset = 300, 255, 85
draw.ellipse([cx - r_outer, cy - r_outer, cx + r_outer, cy + r_outer], fill=ACCENT + (255,))
draw.ellipse(
    [cx - r_inner + offset, cy - r_inner - offset, cx + r_inner + offset, cy + r_inner - offset],
    fill=(0, 0, 0, 0),
)

# Five-pointed star.
star_r = 62
sx, sy = cx + 250, cy - 240
pts = []
for i in range(5):
    for scale in (star_r, star_r * 0.4):
        a = math.radians(i * 72 + (0 if scale == star_r else 36) - 90)
        pts.append((sx + scale * math.cos(a), sy + scale * math.sin(a)))
draw.polygon(pts, fill=ACCENT + (255,))

# Subtle accent ring for definition.
draw.rounded_rectangle([M, M, SIZE - M, SIZE - M], radius=RADIUS, outline=ACCENT + (70,), width=10)

img.save("app-icon.png")
print("saved app-icon.png")
