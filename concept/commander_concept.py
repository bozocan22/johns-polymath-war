#!/usr/bin/env python3
"""Concept still: the commander (player character) of John Kingdom Game,
rendered in the game's low-poly visual language + client HUD, as an
in-game screenshot mock. Palette matches jk_client/tp.rs exactly."""
from PIL import Image, ImageDraw, ImageFont

W, H = 1600, 1000
img = Image.new("RGB", (W, H))
d = ImageDraw.Draw(img, "RGBA")

# ------------------------------------------------------------------ palette
SKY_TOP = (110, 125, 150)
SKY_BOT = (155, 165, 185)
GROUND = (72, 78, 58)
GROUND_L = (90, 100, 72)
BLUE = (85, 125, 215)        # side A torso
BLUE_SHIELD = (150, 175, 230)
RED = (200, 75, 65)          # side B torso
RED_SHIELD = (225, 150, 135)
GOLD = (235, 190, 70)        # commander
GOLD_D = (190, 145, 40)
SKIN = (205, 170, 140)
IRON = (120, 125, 135)
IRON_D = (85, 90, 100)
DARK = (40, 38, 34)
CLOAK = (140, 45, 45)
CLOAK_D = (110, 32, 32)
WOOD = (110, 82, 55)

HORIZON = int(H * 0.45)

# ------------------------------------------------------------------- sky
for y in range(HORIZON):
    t = y / HORIZON
    d.line([(0, y), (W, y)], fill=tuple(int(a + (b - a) * t) for a, b in zip(SKY_TOP, SKY_BOT)))
for y in range(HORIZON, H):
    t = (y - HORIZON) / (H - HORIZON)
    d.line([(0, y), (W, y)], fill=tuple(int(a + (b - a) * t) for a, b in zip(GROUND, GROUND_L)))
# ground stripes for parallax (like the demo grid)
for i in range(18):
    y = HORIZON + int((i / 18) ** 1.8 * (H - HORIZON))
    d.line([(0, y), (W, y)], fill=(62, 68, 50), width=1)

def fog(c, f):
    return tuple(int(a * (1 - f) + b * f) for a, b in zip(c, SKY_BOT))

# ----------------------------------------------- enemy line (distant, red)
def soldier_far(cx, base_y, s, f):
    # shadow
    d.ellipse([cx - 18 * s, base_y - 5 * s, cx + 18 * s, base_y + 5 * s],
              fill=fog((52, 58, 44), f))
    # spear behind
    d.line([(cx + 10 * s, base_y - 120 * s), (cx + 12 * s, base_y)],
           fill=fog(WOOD, f), width=max(1, int(2 * s)))
    d.polygon([(cx + 10 * s, base_y - 130 * s), (cx + 6 * s, base_y - 116 * s),
               (cx + 15 * s, base_y - 117 * s)], fill=fog(IRON, f))
    # torso
    d.rectangle([cx - 12 * s, base_y - 62 * s, cx + 12 * s, base_y], fill=fog(RED, f))
    # head + iron cap
    d.rectangle([cx - 6 * s, base_y - 76 * s, cx + 6 * s, base_y - 62 * s], fill=fog(SKIN, f))
    d.rectangle([cx - 7 * s, base_y - 80 * s, cx + 7 * s, base_y - 72 * s], fill=fog(IRON, f))
    # shield facing us (covers lower torso)
    d.rectangle([cx - 17 * s, base_y - 50 * s, cx + 17 * s, base_y - 6 * s],
                fill=fog(RED_SHIELD, f), outline=fog(DARK, f))
    d.ellipse([cx - 4 * s, base_y - 31 * s, cx + 4 * s, base_y - 23 * s], fill=fog(IRON, f))

# enemy wall: three ranks, rear to front (spear forest builds up)
for y, s, f, x0, n, dx in [
    (HORIZON + 96, 0.85, 0.60, 120, 16, 96),
    (HORIZON + 128, 1.0, 0.50, 168, 15, 100),
    (HORIZON + 168, 1.2, 0.40, 120, 14, 108),
]:
    for i in range(n):
        soldier_far(x0 + i * dx, y, s, f)

# ------------------------------------------- own front rank flanking (blue)
def soldier_near(cx, base_y, s, f=0.12):
    # seen from behind: torso over their own shield (shield mostly hidden)
    d.rectangle([cx - 26 * s, base_y - 130 * s, cx + 26 * s, base_y], fill=fog(BLUE, f))
    d.rectangle([cx - 13 * s, base_y - 162 * s, cx + 13 * s, base_y - 132 * s], fill=fog(SKIN, f))
    # helmet cap
    d.rectangle([cx - 14 * s, base_y - 168 * s, cx + 14 * s, base_y - 152 * s], fill=fog(IRON, f))
    # shield edges peeking around the torso
    d.rectangle([cx - 34 * s, base_y - 108 * s, cx - 26 * s, base_y - 24 * s], fill=fog(BLUE_SHIELD, f))
    d.rectangle([cx + 26 * s, base_y - 108 * s, cx + 34 * s, base_y - 24 * s], fill=fog(BLUE_SHIELD, f))
    # spear
    d.line([(cx + 30 * s, base_y - 260 * s), (cx + 34 * s, base_y - 40 * s)],
           fill=fog(WOOD, f), width=max(2, int(3 * s)))
    d.polygon([(cx + 30 * s, base_y - 274 * s), (cx + 26 * s, base_y - 252 * s),
               (cx + 35 * s, base_y - 254 * s)], fill=fog(IRON, f))

for cx, s in [(210, 1.5), (470, 1.62), (1180, 1.62), (1435, 1.5)]:
    soldier_near(cx, HORIZON + 430, s)

# ---------------------------------------------------- banner bearer (left)
bx, by = 660, HORIZON + 300
d.line([(bx, by - 560), (bx, by - 40)], fill=WOOD, width=8)
d.polygon([(bx + 4, by - 556), (bx + 190, by - 520), (bx + 150, by - 488),
           (bx + 190, by - 456), (bx + 4, by - 430)], fill=CLOAK, outline=CLOAK_D)
# device on banner: gold circle-cross
d.ellipse([bx + 60, by - 522, bx + 110, by - 472], outline=GOLD, width=6)
d.line([(bx + 85, by - 528), (bx + 85, by - 466)], fill=GOLD, width=6)

# ------------------------------------------------------- THE COMMANDER ---
cx, gy = 880, HORIZON + 445          # feet anchor
S = 1.0

def R(x0, y0, x1, y1, c, o=None):
    d.rectangle([cx + x0 * S, gy + y0 * S, cx + x1 * S, gy + y1 * S], fill=c, outline=o)

# cloak behind
d.polygon([(cx - 66, gy - 320), (cx + 60, gy - 320), (cx + 96, gy - 40),
           (cx - 100, gy - 40)], fill=CLOAK, outline=CLOAK_D)
# legs
R(-40, -110, -8, 0, (60, 58, 66))
R(8, -110, 40, 0, (60, 58, 66))
R(-46, -14, -2, 4, DARK)   # boots
R(2, -14, 46, 4, DARK)
# mail skirt + torso (gold-trimmed commander tabard over mail)
R(-52, -240, 52, -100, IRON)                       # mail
for yy in range(-236, -104, 10):                   # mail texture rows
    d.line([(cx - 50, gy + yy), (cx + 50, gy + yy)], fill=IRON_D, width=2)
R(-52, -250, 52, -232, GOLD)                       # shoulder trim
R(-16, -240, 16, -100, GOLD)                       # tabard stripe
R(-52, -132, 52, -116, (110, 82, 55))              # belt
d.ellipse([cx - 10, gy - 132, cx + 10, gy - 112], fill=GOLD)  # buckle
# right arm holding spear
R(40, -240, 74, -150, IRON)
R(52, -160, 84, -130, SKIN)
# spear (tall, iron head)
d.line([(cx + 68, gy - 560), (cx + 68, gy - 20)], fill=WOOD, width=10)
d.polygon([(cx + 68, gy - 610), (cx + 52, gy - 552), (cx + 84, gy - 552)], fill=IRON, outline=IRON_D)
d.rectangle([cx + 62, gy - 552, cx + 74, gy - 540], fill=GOLD)  # collar
# head
R(-22, -300, 22, -252, SKIN)
d.line([(cx - 10, gy - 270), (cx - 2, gy - 270)], fill=DARK, width=3)  # eye
d.line([(cx + 4, gy - 270), (cx + 12, gy - 270)], fill=DARK, width=3)
# helmet: iron dome + gold brow band + fore-aft crest
d.pieslice([cx - 26, gy - 330, cx + 26, gy - 278], 180, 360, fill=IRON, outline=IRON_D)
R(-26, -308, 26, -296, GOLD)
d.polygon([(cx - 6, gy - 366), (cx + 6, gy - 366), (cx + 10, gy - 322), (cx - 10, gy - 322)],
          fill=CLOAK, outline=CLOAK_D)                     # crest
# nasal
R(-4, -296, 4, -270, IRON)
# left arm + ROUND SHIELD (game shield = the wall)
sx, sy, sr = cx - 96, gy - 180, 96
d.ellipse([sx - sr, sy - sr, sx + sr, sy + sr], fill=BLUE_SHIELD, outline=DARK, width=6)
d.ellipse([sx - sr + 12, sy - sr + 12, sx + sr - 12, sy + sr - 12], outline=(120, 145, 205), width=4)
# painted device: gold quarters
d.pieslice([sx - sr + 14, sy - sr + 14, sx + sr - 14, sy + sr - 14], 315, 45, fill=GOLD)
d.pieslice([sx - sr + 14, sy - sr + 14, sx + sr - 14, sy + sr - 14], 135, 225, fill=GOLD)
# iron boss
d.ellipse([sx - 26, sy - 26, sx + 26, sy + 26], fill=IRON, outline=IRON_D, width=4)
d.ellipse([sx - 10, sy - 10, sx + 10, sy + 10], fill=(150, 155, 165))

# player marker (the in-game gold diamond above the commander)
mk_y = gy - 420
d.polygon([(cx, mk_y - 26), (cx + 16, mk_y), (cx, mk_y + 26), (cx - 16, mk_y)],
          fill=(255, 235, 140), outline=GOLD_D)

# ------------------------------------------------------------- HUD (client)
try:
    F = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf", 27)
    Fs = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", 22)
    Ft = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 30)
except OSError:
    F = Fs = Ft = ImageFont.load_default()

d.rectangle([0, 0, W, 128], fill=(15, 16, 20, 150))
d.text((24, 14), "ORDER: Brace   [1 Advance  2 Hold  3 Brace  4 Charge  5 Rotate]", font=F, fill=(255, 255, 255))
d.text((24, 52), "you: stamina  86%   chest load 1840 N", font=Fs, fill=(255, 255, 255))
d.text((24, 84), "wall: 38 vs 37   cohesion 0.33   press 19.2 kN", font=Fs, fill=(255, 255, 255))
d.text((W - 500, 52), "spear READY   kills 2   armor Mail", font=Fs, fill=(255, 235, 140))
d.rectangle([0, H - 54, W, H], fill=(15, 16, 20, 150))
d.text((24, H - 44), "WASD move   SHIFT shoulder-in   LMB thrust   mouse look", font=Fs, fill=(190, 190, 190))
d.text((W - 560, H - 46), "JOHN KINGDOM GAME — commander concept", font=Fs, fill=(150, 150, 160))

out = "/home/user/NewFable/projects/john_kingdom_game/output/concept_commander.png"
img.save(out)
print("wrote", out)
