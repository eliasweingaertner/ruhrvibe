# Ruhrvibe

![Ruhrvibe Logo](ruhrvibe-logo.png)

**A subtractive synthesizer that technically works.** (Human Edit: And that sounds better than you might think!)

## What is this?

This is a vibecoding experiment — a VST3/CLAP synthesizer plugin written in Rust, built entirely through conversation with an AI. No human mass-produced this code by hand. Whether that's impressive or terrifying is left as an exercise for the listener.

The name is a nod to the Ruhr area. The logo was made by another AI. It's AIs all the way down.

## What can it do?

Quite a bit, actually. Don't tell anyone we were surprised too.

- **2 oscillators** per voice — sine, saw, square, triangle, and noise (for when you want your music to sound like a broken radio)
- **Unison detuning with stereo spread** — up to 7 slightly-out-of-tune copies per oscillator, fanned across the stereo field, because one saw wave is never enough
- **2 filters in series** — lowpass, highpass, bandpass, notch; each with resonance, drive, and envelope modulation. Cytomic-style state variable filter that we definitely understood on the first try
- **4 envelopes** — amplitude, two filter envelopes, and a pitch envelope for kicks that go *bwoooom*
- **Polyphony** — up to 32 voices, with voice stealing for when you hold down too many keys
- **An effects chain** — chorus, ping-pong delay, octave-up shimmer, and a host-synced trance gate. We got carried away
- **An arpeggiator with scale-lock and scale-walk** — hold one key, get a full scale run; hold a chord, keep it in key across octaves
- **~119 factory presets** across 11 categories — from Fat Bass to Haunted Hall to Alf, each one a best guess at what those things are supposed to sound like
- **An actual GUI** — with colors and knobs and everything. It even scales on HiDPI displays (after three attempts)

## What can't it do?

LFOs. Wavetables. Modulation matrix. FM. Sidechain. Anything a *professional* synthesizer would quietly list under "of course it does that." But hey, it makes sound come out, and sometimes that sound is even pleasant.

## Hear it first

- [**ruhrvibe-demo.mp3**](ruhrvibe-demo.mp3?raw=true) — a short rendered demo. Listen before you build.
- [**ruhrvibe-demo.xrns**](ruhrvibe-demo.xrns) — the Renoise project that produced it. Open it in Renoise with the plugin installed to poke at how the sounds were made.

## User guide

Every knob, every preset, every signal path is documented in [**USER_GUIDE.md**](USER_GUIDE.md). Start there if you want to know what any of this actually does, or skim the recipes section if you just want to make noise quickly.

## Building

```bash
# You'll need Rust installed
cargo xtask bundle ruhrvibe --release

# On macOS, build a universal (x86_64 + arm64) bundle instead:
cargo xtask bundle-universal ruhrvibe --release
```

Output lands in `target/bundled/` as both `.vst3` and `.clap` bundles.

**Installing (Windows):**

```bash
cp -r "target/bundled/ruhrvibe.vst3" "/c/Program Files/Common Files/VST3/"
```

Or grab a prebuilt bundle from GitHub Actions — CI produces Linux, Windows, and macOS artifacts on every commit to `master`.

**Tested in:** REAPER and Renoise. Other VST3/CLAP hosts (Bitwig, Live, Studio One, FL) should work — they just haven't been verified.

## Tech stack

- [nih-plug](https://github.com/robbert-vdh/nih-plug) — the only Rust VST3 framework that exists, conveniently also the best one
- [nih_plug_vizia](https://github.com/robbert-vdh/nih-plug) — for the GUI, because declarative reactive UI in Rust is apparently a thing now
- PolyBLEP anti-aliasing — so the saw waves don't sound like angry bees
- Fast exp2 approximation — because `powf()` 200 times per sample was a lifestyle choice we reconsidered

## License

GPL-3.0-or-later. See [LICENSE.md](LICENSE.md) for the full text.

Translated from lawyer: modify and share freely, but any redistribution has
to stay GPL-compatible and include source. No quietly bundling this into a
closed-source commercial product.
