#!/usr/bin/env python3
"""Procedural SFX generator for jk_tdm — pure stdlib, deterministic.

Run from this directory: python3 gen_sfx.py
Writes the .wav files the client loads (see main.rs SfxAssets).
Regenerate + commit whenever a sound is tuned; the game only reads the WAVs.
"""
import math
import random
import struct
import wave

SR = 22050
rng = random.Random(0xC0B0)  # fixed seed — as close to COWBOY as hex allows


def write(name, samples, gain=0.9):
    peak = max(1e-9, max(abs(s) for s in samples))
    k = gain / peak
    with wave.open(name, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes(
            b"".join(
                struct.pack("<h", int(max(-1.0, min(1.0, s * k)) * 32767))
                for s in samples
            )
        )
    print(f"wrote {name} ({len(samples) / SR * 1000:.0f} ms)")


def n_samples(sec):
    return int(sec * SR)


def noise_burst(sec, tau, lp=0.25):
    """Exponentially decaying filtered noise. lp in (0,1]: 1 = no filtering."""
    out, y = [], 0.0
    for i in range(n_samples(sec)):
        y += lp * (rng.uniform(-1, 1) - y)  # one-pole lowpass
        out.append(y * math.exp(-i / (tau * SR)))
    return out

def sine(sec, f0, tau=None, f1=None, vib=0.0):
    out, ph = [], 0.0
    n = n_samples(sec)
    for i in range(n):
        t = i / SR
        f = f0 if f1 is None else f0 + (f1 - f0) * i / n
        ph += 2 * math.pi * f / SR
        env = 1.0 if tau is None else math.exp(-t / tau)
        out.append(math.sin(ph + vib * math.sin(2 * math.pi * 30 * t) * math.exp(-t / 0.06)) * env)
    return out

def mix(*tracks):
    n = max(len(t) for t in tracks)
    return [sum(t[i] for t in tracks if i < len(t)) for i in range(n)]

def cat(*tracks, gap=0.0):
    out = []
    for t in tracks:
        out.extend(t)
        out.extend([0.0] * n_samples(gap))
    return out

def scaled(t, k):
    return [s * k for s in t]

def delay(t, sec, k):
    """t, delayed by `sec` and scaled by `k` — one echo tap."""
    return [0.0] * n_samples(sec) + scaled(t, k)


# gunshots, v7: every shot is now FOUR layers, not two, because a real
# gunshot is not "noise over a thump" — it is a supersonic CRACK (the
# transient that makes a gun sound dangerous rather than percussive),
# a BODY (the muzzle blast, what v6 had as its noise_burst), a SUB
# thump (the mechanical/pressure impulse, low sine, scaled to the
# cartridge), and — for anything with real barrel length — a TAIL: the
# blast bouncing back off the environment a beat later. Handguns stay
# tight and dry (a pistol crack dies where it's born); rifles get one
# slap-back; the sniper's magnum round gets two, spaced further out,
# because that's the "boom...boom" a big rifle makes across open ground.
def crack(sec, lp, tau=0.006):
    """The supersonic transient: near-full-spectrum noise, gone almost
    instantly. This is the layer v6 never had, and it's the difference
    between a shot sounding dangerous and sounding like a cap gun."""
    return noise_burst(sec, tau, lp)

def gunshot(crack_lp, body_sec, body_tau, body_lp, sub_f, sub_tau, sub_gain,
            tail=None):
    layers = [
        scaled(crack(0.03, crack_lp), 1.0),
        noise_burst(body_sec, body_tau, body_lp),
        scaled(sine(body_sec * 0.85, sub_f, sub_tau), sub_gain),
    ]
    if tail:
        for gap, k, f in tail:
            layers.append(delay(scaled(sine(0.22, f, 0.09), k), gap, 1.0))
            layers.append(delay(scaled(noise_burst(0.10, 0.03, 0.30), k * 0.6), gap, 1.0))
    return mix(*layers)

# handguns: tight and dry — the crack is bright and short, the body is
# small, no tail (a pistol round doesn't carry across a field). Bigger
# calibre = lower sub thump + longer body, same dry ending.
write("shot_handgun.wav", gunshot(0.85, 0.09, 0.017, 0.42, 380, 0.03, 0.55))
write("shot_glock.wav", gunshot(0.90, 0.08, 0.015, 0.46, 420, 0.025, 0.5))
write("shot_deagle.wav", gunshot(0.70, 0.16, 0.032, 0.28, 150, 0.075, 1.0))
write("shot_mp5.wav", gunshot(0.95, 0.06, 0.012, 0.44, 360, 0.018, 0.45))
write("shot_shotgun.wav", gunshot(0.55, 0.22, 0.050, 0.20, 130, 0.10, 1.05))
# rifles: sharper crack than any handgun (supersonic round), a punchier
# mid body, and one slap-back off the ground/terrain a beat later — the
# thing that makes automatic rifle fire sound like it's happening
# somewhere rather than in a booth. AK is dirtier/lower than the M4.
write("shot_ak.wav", gunshot(0.65, 0.11, 0.024, 0.32, 200, 0.055, 0.85,
                              tail=[(0.075, 0.18, 130)]))
write("shot_rifle.wav", gunshot(0.75, 0.11, 0.022, 0.36, 250, 0.045, 0.7,
                                 tail=[(0.070, 0.16, 160)]))
write("shot_mg.wav", gunshot(0.80, 0.07, 0.014, 0.40, 290, 0.028, 0.55))
# sniper: the sharpest crack of the roster (a magnum round breaking the
# sound barrier close to the ear), the longest low body, and TWO
# slap-backs spaced further apart than the rifles' — the "boom...boom"
# of a big rifle report bouncing back across open ground.
write("shot_sniper.wav", gunshot(0.60, 0.34, 0.070, 0.18, 100, 0.14, 1.15,
                                  tail=[(0.11, 0.30, 90), (0.24, 0.14, 70)]))
# dry fire: a tiny mechanical tick
write("click.wav", cat(noise_burst(0.012, 0.003, 0.9), noise_burst(0.018, 0.004, 0.9), gap=0.03), gain=0.4)
# a round slapping the shield plate: metallic clang with ring
write("shield.wav", mix(scaled(sine(0.16, 620, 0.05, vib=9.0), 0.9), scaled(sine(0.12, 940, 0.04), 0.5), scaled(noise_burst(0.03, 0.008, 0.6), 0.6)))
write("bow.wav", mix(scaled(sine(0.28, 170, 0.08, vib=6.0), 1.0), scaled(noise_burst(0.03, 0.008, 0.6), 0.5)))

# spear: a whoosh — noise through a swelling envelope
_wh = noise_burst(0.32, 0.5, 0.12)
write("spear.wav", [s * math.sin(math.pi * i / len(_wh)) for i, s in enumerate(_wh)])

write("hit.wav", sine(0.07, 1300, 0.018))

# dodge roll: a short cloth-and-motion tumble — swelling low noise
_rl = noise_burst(0.30, 0.4, 0.10)
write("roll.wav", [s * math.sin(math.pi * i / len(_rl)) for i, s in enumerate(_rl)], gain=0.5)

write("headshot.wav", cat(sine(0.06, 900, 0.02), sine(0.10, 1500, 0.03)))
write("hurt.wav", mix(scaled(sine(0.14, 150, 0.05), 1.0), scaled(noise_burst(0.06, 0.02, 0.3), 0.4)))
write("pickup.wav", cat(sine(0.08, 620, 0.05), sine(0.12, 930, 0.06)))
write("reload.wav", cat(noise_burst(0.02, 0.005, 0.8), noise_burst(0.03, 0.007, 0.8), gap=0.10), gain=0.55)
write("jump.wav", scaled(sine(0.12, 220, 0.05, f1=340), 0.7), gain=0.4)
write("kill.wav", sine(0.22, 700, 0.09, f1=420))
write("win.wav", cat(sine(0.15, 523, 0.09), sine(0.15, 659, 0.09), sine(0.30, 784, 0.14)))

# ---------------------------------------------------------------------
# THREAT LOCK WARNING — the mech sensor's "someone has a clear shot on
# you" cue. Added last, and built from `sine` ONLY, deliberately: every
# other voice in this file draws from the shared `rng` via `noise_burst`,
# so a new noise layer anywhere above would shift the stream and silently
# re-roll unrelated sounds. Pure sine touches no RNG, so every existing
# .wav stays byte-identical when this file is regenerated.
#
# Design constraints, from the owner's brief: it must be RARE and QUIET
# ("do NOT make the HUD excessively intrusive"), and it must not be
# confusable with any existing cue. `click.wav` already means "empty
# magazine" and everything else is a gunshot, a hit, a pickup or a
# jingle — so this is deliberately the only ELECTRONIC voice in the
# bank: two clean rising pips, no noise, no percussive attack. A rise
# reads as a question being asked of you (a lock acquiring); a fall
# would read as something completing or dying.
#
# Kept short (~0.16 s total) and at low gain so it can fire during a
# firefight without stepping on the gunfire it is warning you about.
write(
    "threat_lock.wav",
    cat(
        mix(sine(0.055, 1046.5, 0.020), scaled(sine(0.055, 2093.0, 0.014), 0.22)),
        mix(sine(0.070, 1396.9, 0.026), scaled(sine(0.070, 2793.8, 0.018), 0.22)),
        gap=0.030,
    ),
    gain=0.34,
)
