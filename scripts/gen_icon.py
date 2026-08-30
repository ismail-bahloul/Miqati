"""Generate the app icon for Salaat Widget.

Draws a gold crescent on a deep midnight-blue rounded background, echoing the
widget's design tokens (midnight blue #1C2541, gold #F5A623).
"""
from PIL import Image, ImageDraw

SIZE = 1024
blue = (28, 37, 65)      # #1C2541
gold = (245, 166, 35)    # #F5A623

img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

# Rounded-square background (full-bleed square is fine too).
margin = 20
draw.rounded_rectangle(
    [margin, margin, SIZE - margin, SIZE - margin],
    radius=180,
    fill=blue + (255,),
)

# Crescent = a filled circle minus an overlapping offset circle.
cx, cy = SIZE // 2, SIZE // 2
r_outer = 300
r_inner = 255
offset = 85  # how much the cut-out circle shifts to form the crescent

draw.ellipse([cx - r_outer, cy - r_outer, cx + r_outer, cy + r_outer], fill=gold + (255,))
draw.ellipse(
    [
        cx - r_inner + offset,
        cy - r_inner - offset,
        cx + r_inner + offset,
        cy + r_inner - offset,
    ],
    fill=(0, 0, 0, 0),
)

# Small star/point detail for character (optional).
star_r = 60
sx, sy = cx + 250, cy - 240
try:
    from math import cos, radians as rad, sin
    pts = []
    for i in range(5):
        for scale in (star_r, star_r * 0.4):
            a = rad(i * 72 + (0 if scale == star_r else 36) - 90)
            pts.append((sx + scale * cos(a), sy + scale * sin(a)))
    draw.polygon(pts, fill=gold + (255,))
except Exception:
    pass

img.save("app-icon.png")
print("saved app-icon.png")
