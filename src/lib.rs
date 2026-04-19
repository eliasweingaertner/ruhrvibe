//! Ruhrvibe — Subtractive Synthesizer VST3 Plugin
//!
//! A polyphonic subtractive synthesizer built with nih-plug and Iced.
//! Features:
//! - 2 oscillators per voice (sine/saw/square/triangle/noise) with PolyBLEP anti-aliasing
//! - Unison detuning (up to 7 voices per oscillator) for harmonic richness
//! - 2-slot series filter chain per voice (LP/HP/BP/Notch)
//! - Amp envelope + 2 filter envelopes + pitch envelope
//! - Configurable polyphony (1-32 voices) with voice stealing
//! - Vizia-based GUI with factory presets (melodic + drums)

mod envelope;
mod editor;
mod fast_math;
mod filter;
mod fx;
mod oscillator;
mod params;
mod presets;
mod synth;
mod voice;

use synth::SubtractiveSynth;

nih_plug::nih_export_vst3!(SubtractiveSynth);
nih_plug::nih_export_clap!(SubtractiveSynth);
