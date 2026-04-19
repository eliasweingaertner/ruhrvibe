# Ruhrvibe

![Ruhrvibe Logo](ruhrvibe-logo.png)

**A subtractive synthesizer that technically works.**

## What is this?

This is a vibecoding experiment — a VST3/CLAP synthesizer plugin written in Rust, built entirely through conversation with an AI. No human mass-produced this code by hand. Whether that's impressive or terrifying is left as an exercise for the listener.

The name is a nod to the Ruhr area. The logo was made by another AI. It's AIs all the way down.

## What can it do?

Quite a bit, actually. Don't tell anyone we were surprised too.

- **2 oscillators** per voice — sine, saw, square, triangle, and noise (for when you want your music to sound like a broken radio)
- **Unison detuning** — stack up to 7 slightly-out-of-tune copies per oscillator, because one saw wave is never enough
- **2 filters in series** — low-pass, high-pass, band-pass, notch, each with resonance, drive, and envelope modulation. Uses a Cytomic-style state variable filter that we definitely understood on the first try
- **4 envelopes** — amplitude, 2 filter envelopes, and a pitch envelope for kicks that go *bwoooom*
- **Polyphony** — up to 32 voices, with voice stealing for when you hold down too many keys
- **16 factory presets** — from "Fat Bass" to "Hi-Hat", each one a best guess at what those things are supposed to sound like
- **An actual GUI** — with colors and knobs and everything. It even scales on HiDPI displays (after three attempts)

## What can't it do?

Effects. Stereo spread. LFOs. Wavetables. Anything a professional synthesizer would consider table stakes. But hey, it makes sound come out, and sometimes that sound is even pleasant.

## Building

```bash
# You'll need Rust installed
cargo build --release

# Bundle as VST3 + CLAP (Windows)
./bundle.sh

# Install
cp -r "target/bundled/Ruhrvibe.vst3" "/c/Program Files/Common Files/VST3/"
```

## Tech stack

- [nih-plug](https://github.com/robbert-vdh/nih-plug) — the only Rust VST3 framework that exists, conveniently also the best one
- [nih_plug_iced](https://github.com/robbert-vdh/nih-plug) — for the GUI, using a version of Iced old enough to have character
- PolyBLEP anti-aliasing — so the saw waves don't sound like angry bees
- Fast exp2 approximation — because `powf()` 200 times per sample was a lifestyle choice we reconsidered

## License

GPL-3.0-or-later. See [LICENSE.md](LICENSE.md) for the full text.

Translated from lawyer: modify and share freely, but any redistribution has
to stay GPL-compatible and include source. No quietly bundling this into a
closed-source commercial product.
