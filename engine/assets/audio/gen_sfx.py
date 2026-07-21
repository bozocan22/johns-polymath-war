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


# gunshots: noise crack + low thump, sized to the gun (v6: the full
# roster gets ITS OWN voice — pitch, length, and punch per weapon)
write("shot_handgun.wav", mix(noise_burst(0.10, 0.020, 0.45), scaled(sine(0.09, 400, 0.03), 0.5)))
write("shot_glock.wav", mix(noise_burst(0.09, 0.018, 0.50), scaled(sine(0.08, 430, 0.025), 0.45)))
write("shot_deagle.wav", mix(noise_burst(0.18, 0.038, 0.30), scaled(sine(0.16, 170, 0.06), 0.85)))
write("shot_mp5.wav", mix(noise_burst(0.07, 0.014, 0.48), scaled(sine(0.06, 380, 0.02), 0.4)))
write("shot_shotgun.wav", mix(noise_burst(0.24, 0.055, 0.22), scaled(sine(0.20, 140, 0.08), 0.9)))
write("shot_ak.wav", mix(noise_burst(0.12, 0.026, 0.34), scaled(sine(0.10, 220, 0.045), 0.7)))
write("shot_rifle.wav", mix(noise_burst(0.13, 0.028, 0.38), scaled(sine(0.11, 260, 0.04), 0.6)))
write("shot_mg.wav", mix(noise_burst(0.08, 0.016, 0.42), scaled(sine(0.07, 300, 0.025), 0.5)))
write("shot_sniper.wav", mix(noise_burst(0.38, 0.075, 0.20), scaled(sine(0.30, 110, 0.10), 0.9)))
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
