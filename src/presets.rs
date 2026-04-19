//! Factory presets.
//!
//! Presets are defined as compile-time constant structs containing concrete
//! parameter values. On selection, `apply_preset` writes each value through
//! the `GuiContext` so DAW automation is notified of the changes.
//!
//! Includes approximations of all 128 General MIDI instruments, plus some
//! fun extras and 8-bit presets. A subtractive synth can only do so much —
//! don't expect a convincing sitar.

use nih_plug::prelude::*;
use std::sync::Arc;

use crate::params::{ArpPattern, FilterType, SyncRate, SynthParams, Waveform};

#[derive(Clone, Copy)]
pub struct OscPreset {
    pub waveform: Waveform,
    pub level: f32,
    pub detune: f32,
    pub octave: i32,
    pub enabled: bool,
    pub unison_voices: i32,
    pub unison_spread: f32,
    pub pan: f32,
    pub stereo_spread: f32,
}

#[derive(Clone, Copy)]
pub struct FilterPreset {
    pub filter_type: FilterType,
    pub cutoff: f32,
    pub resonance: f32,
    pub drive: f32,
    pub env_amount: f32,
    pub enabled: bool,
}

#[derive(Clone, Copy)]
pub struct EnvPreset {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

#[derive(Clone, Copy)]
pub struct PitchEnvPreset {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub amount: f32,
}

#[derive(Clone, Copy)]
pub struct ChorusPreset {
    pub rate: f32,
    pub depth: f32,
    pub mix: f32,
    pub enabled: bool,
}

#[derive(Clone, Copy)]
pub struct DelayPreset {
    pub time_ms: f32,
    pub feedback: f32,
    pub tone: f32,
    pub mix: f32,
    pub enabled: bool,
}

#[derive(Clone, Copy)]
pub struct ShimmerPreset {
    pub time_ms: f32,
    pub feedback: f32,
    pub mix: f32,
    pub enabled: bool,
}

#[derive(Clone, Copy)]
pub struct GapperPreset {
    pub rate: SyncRate,
    pub duty: f32,
    pub smooth: f32,
    pub depth: f32,
    pub enabled: bool,
}

#[derive(Clone, Copy)]
pub struct FxPreset {
    pub chorus: ChorusPreset,
    pub delay: DelayPreset,
    pub shimmer: ShimmerPreset,
    pub gapper: GapperPreset,
}

#[derive(Clone, Copy)]
pub struct ArpPreset {
    pub pattern: ArpPattern,
    pub rate: SyncRate,
    pub octaves: i32,
    pub gate: f32,
    pub enabled: bool,
}

#[derive(Clone, Copy)]
pub struct Preset {
    pub name: &'static str,
    pub category: &'static str,
    pub osc1: OscPreset,
    pub osc2: OscPreset,
    pub filter1: FilterPreset,
    pub filter2: FilterPreset,
    pub amp_env: EnvPreset,
    pub filter1_env: EnvPreset,
    pub filter2_env: EnvPreset,
    pub pitch_env: PitchEnvPreset,
    pub master_gain_db: f32,
    pub fx: FxPreset,
    pub arp: ArpPreset,
}

// -----------------------------------------------------------------------
// Shorthand constructors
// -----------------------------------------------------------------------

const fn o(w: Waveform, lv: f32, det: f32, oct: i32, en: bool, uni: i32, sp: f32) -> OscPreset {
    // Default stereo: center, 50% unison spread (only audible when unison > 1).
    OscPreset { waveform: w, level: lv, detune: det, octave: oct, enabled: en,
                unison_voices: uni, unison_spread: sp, pan: 0.0, stereo_spread: 0.5 }
}
/// Oscillator preset with explicit pan and stereo spread.
#[allow(dead_code)]
const fn op(w: Waveform, lv: f32, det: f32, oct: i32, en: bool, uni: i32, sp: f32,
            pan: f32, st: f32) -> OscPreset {
    OscPreset { waveform: w, level: lv, detune: det, octave: oct, enabled: en,
                unison_voices: uni, unison_spread: sp, pan, stereo_spread: st }
}
const fn f(ft: FilterType, cut: f32, res: f32, drv: f32, ea: f32, en: bool) -> FilterPreset {
    FilterPreset { filter_type: ft, cutoff: cut, resonance: res, drive: drv, env_amount: ea, enabled: en }
}
const fn e(a: f32, d: f32, s: f32, r: f32) -> EnvPreset {
    EnvPreset { attack: a, decay: d, sustain: s, release: r }
}
const fn pe(a: f32, d: f32, s: f32, r: f32, amt: f32) -> PitchEnvPreset {
    PitchEnvPreset { attack: a, decay: d, sustain: s, release: r, amount: amt }
}

// Waveform aliases for readability
const SIN: Waveform = Waveform::Sine;
const SAW: Waveform = Waveform::Saw;
const SQR: Waveform = Waveform::Square;
const TRI: Waveform = Waveform::Triangle;
const NOI: Waveform = Waveform::Noise;

// Filter type aliases
const LP: FilterType = FilterType::LowPass;
const HP: FilterType = FilterType::HighPass;
const BP: FilterType = FilterType::BandPass;
const NT: FilterType = FilterType::Notch;

// Common filter/envelope defaults
const FOFF: FilterPreset = f(LP, 12000.0, 0.0, 1.0, 0.0, false);
const EOFF: EnvPreset = e(0.01, 0.3, 0.5, 0.3);
const POFF: PitchEnvPreset = pe(0.001, 0.1, 0.0, 0.05, 0.0);
const OOFF: OscPreset = o(SIN, 0.0, 0.0, 0, false, 1, 0.0);

// FX preset shorthand constructors.
const fn ch(rate: f32, depth: f32, mix: f32, enabled: bool) -> ChorusPreset {
    ChorusPreset { rate, depth, mix, enabled }
}
const fn dl(time_ms: f32, feedback: f32, tone: f32, mix: f32, enabled: bool) -> DelayPreset {
    DelayPreset { time_ms, feedback, tone, mix, enabled }
}
const fn sh(time_ms: f32, feedback: f32, mix: f32, enabled: bool) -> ShimmerPreset {
    ShimmerPreset { time_ms, feedback, mix, enabled }
}
const fn gp(rate: SyncRate, duty: f32, smooth: f32, depth: f32, enabled: bool) -> GapperPreset {
    GapperPreset { rate, duty, smooth, depth, enabled }
}
const fn fx(chorus: ChorusPreset, delay: DelayPreset, shimmer: ShimmerPreset, gapper: GapperPreset) -> FxPreset {
    FxPreset { chorus, delay, shimmer, gapper }
}
const fn ap(pattern: ArpPattern, rate: SyncRate, octaves: i32, gate: f32, enabled: bool) -> ArpPreset {
    ArpPreset { pattern, rate, octaves, gate, enabled }
}

// All-off defaults.
const CHR_OFF: ChorusPreset = ch(0.5, 0.5, 0.35, false);
const DLY_OFF: DelayPreset = dl(350.0, 0.35, 0.6, 0.25, false);
const SHM_OFF: ShimmerPreset = sh(500.0, 0.45, 0.35, false);
const GAP_OFF: GapperPreset = gp(SyncRate::Eighth, 0.5, 0.1, 1.0, false);
const FX_OFF: FxPreset = fx(CHR_OFF, DLY_OFF, SHM_OFF, GAP_OFF);
const ARP_OFF: ArpPreset = ap(ArpPattern::Up, SyncRate::Sixteenth, 1, 0.5, false);

pub const FACTORY_PRESETS: &[Preset] = &[
    // ===================================================================
    // COMMON PRESETS
    // ===================================================================
    Preset { category: "Common", name: "Init",
        osc1: o(SAW, 0.75, 0.0, 0, true, 1, 20.0),
        osc2: o(SQR, 0.0, 0.0, 0, false, 1, 20.0),
        filter1: f(LP, 12000.0, 0.2, 1.0, 0.0, true),
        filter2: FOFF,
        amp_env: e(0.01, 0.3, 0.7, 0.3),
        filter1_env: e(0.01, 0.3, 0.5, 0.3),
        filter2_env: EOFF, pitch_env: POFF, master_gain_db: -6.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Fat Bass",
        osc1: o(SAW, 0.8, -7.0, -1, true, 3, 12.0),
        osc2: o(SAW, 0.7, 7.0, -1, true, 3, 12.0),
        filter1: f(LP, 500.0, 0.6, 1.5, 0.6, true),
        filter2: FOFF,
        amp_env: e(0.005, 0.2, 0.8, 0.15),
        filter1_env: e(0.005, 0.25, 0.3, 0.2),
        filter2_env: EOFF, pitch_env: POFF, master_gain_db: -6.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Warm Pad",
        osc1: o(TRI, 0.7, 0.0, 0, true, 5, 25.0),
        osc2: o(SAW, 0.5, 5.0, 0, true, 5, 30.0),
        filter1: f(LP, 2000.0, 0.2, 1.0, 0.3, true),
        filter2: FOFF,
        amp_env: e(1.2, 0.5, 0.8, 1.8),
        filter1_env: e(1.5, 1.5, 0.5, 1.5),
        filter2_env: EOFF, pitch_env: POFF, master_gain_db: -8.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Pluck",
        osc1: o(SQR, 0.75, 0.0, 0, true, 1, 20.0),
        osc2: OOFF,
        filter1: f(LP, 3000.0, 0.4, 1.2, 0.8, true),
        filter2: FOFF,
        amp_env: e(0.003, 0.25, 0.0, 0.25),
        filter1_env: e(0.003, 0.2, 0.0, 0.2),
        filter2_env: EOFF, pitch_env: POFF, master_gain_db: -6.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Lead",
        osc1: o(SAW, 0.7, -3.0, 0, true, 3, 15.0),
        osc2: o(SQR, 0.5, 3.0, 0, true, 1, 20.0),
        filter1: f(LP, 4500.0, 0.35, 1.3, 0.3, true),
        filter2: FOFF,
        amp_env: e(0.02, 0.2, 0.75, 0.2),
        filter1_env: e(0.05, 0.3, 0.6, 0.3),
        filter2_env: EOFF, pitch_env: POFF, master_gain_db: -6.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Sub Bass",
        osc1: o(SIN, 0.9, 0.0, -2, true, 1, 0.0),
        osc2: OOFF,
        filter1: FOFF,
        filter2: FOFF,
        amp_env: e(0.01, 0.1, 0.9, 0.1),
        filter1_env: EOFF,
        filter2_env: EOFF, pitch_env: POFF, master_gain_db: -4.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Strings",
        osc1: o(TRI, 0.7, -5.0, 0, true, 5, 18.0),
        osc2: o(TRI, 0.7, 5.0, 0, true, 5, 18.0),
        filter1: f(LP, 5000.0, 0.15, 1.0, 0.2, true),
        filter2: FOFF,
        amp_env: e(0.8, 0.4, 0.85, 1.2),
        filter1_env: e(1.2, 0.5, 0.7, 1.2),
        filter2_env: EOFF, pitch_env: POFF, master_gain_db: -8.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Brass",
        osc1: o(SAW, 0.8, 0.0, 0, true, 3, 10.0),
        osc2: o(SAW, 0.4, 8.0, 0, true, 1, 20.0),
        filter1: f(LP, 1500.0, 0.3, 1.2, 0.7, true),
        filter2: FOFF,
        amp_env: e(0.08, 0.3, 0.7, 0.3),
        filter1_env: e(0.05, 0.5, 0.4, 0.3),
        filter2_env: EOFF, pitch_env: POFF, master_gain_db: -6.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Dreamy Pan Flute",
        osc1: o(SIN, 0.8, 0.0, 0, true, 3, 8.0),
        osc2: o(TRI, 0.25, 3.0, 0, true, 3, 10.0),
        filter1: f(LP, 3000.0, 0.25, 1.0, 0.15, true),
        filter2: f(NT, 1800.0, 0.4, 1.0, 0.0, true),
        amp_env: e(0.08, 0.2, 0.7, 0.4),
        filter1_env: e(0.1, 0.5, 0.8, 0.3),
        filter2_env: EOFF, pitch_env: POFF, master_gain_db: -7.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Piano",
        osc1: o(SAW, 0.75, -4.0, 0, true, 2, 6.0),
        osc2: o(SAW, 0.55, 4.0, -1, true, 2, 6.0),
        filter1: f(LP, 1800.0, 0.2, 1.0, 0.85, true),
        filter2: FOFF,
        amp_env: e(0.003, 0.9, 0.15, 0.35),
        filter1_env: e(0.003, 0.35, 0.0, 0.25),
        filter2_env: EOFF, pitch_env: POFF, master_gain_db: -6.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Swoosh",
        osc1: o(SQR, 0.6, 12.0, 0, true, 5, 40.0),
        osc2: o(SAW, 0.6, -12.0, 0, true, 5, 40.0),
        filter1: f(BP, 800.0, 0.7, 1.0, 1.0, true),
        filter2: f(HP, 300.0, 0.5, 1.0, 0.9, true),
        amp_env: e(1.8, 0.5, 0.7, 2.5),
        filter1_env: e(2.0, 0.5, 0.8, 2.2),
        filter2_env: e(2.5, 0.3, 0.9, 2.0),
        pitch_env: POFF, master_gain_db: -8.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Supersaw",
        osc1: o(SAW, 0.8, 0.0, 0, true, 7, 35.0),
        osc2: o(SAW, 0.6, 0.0, 1, true, 7, 40.0),
        filter1: f(LP, 8000.0, 0.15, 1.0, 0.2, true),
        filter2: FOFF,
        amp_env: e(0.02, 0.3, 0.8, 0.5),
        filter1_env: e(0.01, 0.5, 0.6, 0.4),
        filter2_env: EOFF, pitch_env: POFF, master_gain_db: -8.0,
        fx: FX_OFF, arp: ARP_OFF,
    },

    // ===================================================================
    // DRUM KIT
    // ===================================================================
    Preset { category: "Common", name: "Kick",
        osc1: o(SIN, 0.95, 0.0, -1, true, 1, 0.0),
        osc2: OOFF,
        filter1: f(LP, 500.0, 0.0, 1.0, 0.0, true),
        filter2: FOFF,
        amp_env: e(0.001, 0.25, 0.0, 0.15),
        filter1_env: EOFF, filter2_env: EOFF,
        pitch_env: pe(0.001, 0.06, 0.0, 0.05, 36.0),
        master_gain_db: -3.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Snare",
        osc1: o(SIN, 0.7, 0.0, 0, true, 1, 0.0),
        osc2: o(NOI, 0.8, 0.0, 0, true, 1, 0.0),
        filter1: f(BP, 2000.0, 0.3, 1.2, 0.0, true),
        filter2: f(HP, 500.0, 0.1, 1.0, 0.0, true),
        amp_env: e(0.001, 0.15, 0.0, 0.12),
        filter1_env: EOFF, filter2_env: EOFF,
        pitch_env: pe(0.001, 0.04, 0.0, 0.03, 12.0),
        master_gain_db: -4.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Hi-Hat",
        osc1: o(NOI, 0.9, 0.0, 0, true, 1, 0.0),
        osc2: o(SQR, 0.15, 0.0, 2, true, 1, 0.0),
        filter1: f(HP, 7000.0, 0.3, 1.0, 0.0, true),
        filter2: f(BP, 10000.0, 0.5, 1.0, 0.0, true),
        amp_env: e(0.001, 0.06, 0.0, 0.04),
        filter1_env: EOFF, filter2_env: EOFF,
        pitch_env: POFF, master_gain_db: -6.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Tom",
        osc1: o(SIN, 0.9, 0.0, 0, true, 1, 0.0),
        osc2: o(NOI, 0.2, 0.0, 0, true, 1, 0.0),
        filter1: f(LP, 1500.0, 0.2, 1.0, 0.0, true),
        filter2: FOFF,
        amp_env: e(0.001, 0.35, 0.0, 0.2),
        filter1_env: EOFF, filter2_env: EOFF,
        pitch_env: pe(0.001, 0.08, 0.0, 0.05, 24.0),
        master_gain_db: -4.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Common", name: "Clap",
        osc1: o(NOI, 0.95, 0.0, 0, true, 1, 0.0),
        osc2: OOFF,
        filter1: f(BP, 1200.0, 0.5, 1.3, 0.0, true),
        filter2: f(HP, 600.0, 0.2, 1.0, 0.0, true),
        amp_env: e(0.001, 0.18, 0.0, 0.15),
        filter1_env: EOFF, filter2_env: EOFF,
        pitch_env: POFF, master_gain_db: -5.0,
        fx: FX_OFF, arp: ARP_OFF,
    },

    // ===================================================================
    // 8-BIT PRESETS
    // ===================================================================
    Preset { category: "8-Bit", name: "8-Bit Lead",
        osc1: o(SQR, 0.8, 0.0, 0, true, 1, 0.0),
        osc2: OOFF,
        filter1: FOFF,
        filter2: FOFF,
        amp_env: e(0.001, 0.05, 0.90, 0.05),
        filter1_env: EOFF, filter2_env: EOFF, pitch_env: POFF, master_gain_db: -6.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "8-Bit", name: "8-Bit Bass",
        osc1: o(SQR, 0.8, 0.0, -1, true, 1, 0.0),
        osc2: OOFF,
        filter1: FOFF,
        filter2: FOFF,
        amp_env: e(0.001, 0.15, 0.60, 0.08),
        filter1_env: EOFF, filter2_env: EOFF, pitch_env: POFF, master_gain_db: -5.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "8-Bit", name: "8-Bit Arp",
        osc1: o(SQR, 0.7, 0.0, 1, true, 1, 0.0),
        osc2: o(SQR, 0.3, 0.0, 0, true, 1, 0.0),
        filter1: FOFF,
        filter2: FOFF,
        amp_env: e(0.001, 0.08, 0.0, 0.04),
        filter1_env: EOFF, filter2_env: EOFF, pitch_env: POFF, master_gain_db: -6.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "8-Bit", name: "8-Bit Pad",
        osc1: o(SQR, 0.5, -8.0, 0, true, 1, 0.0),
        osc2: o(SQR, 0.5, 8.0, 0, true, 1, 0.0),
        filter1: f(LP, 3000.0, 0.0, 1.0, 0.0, true),
        filter2: FOFF,
        amp_env: e(0.3, 0.2, 0.80, 0.5),
        filter1_env: EOFF, filter2_env: EOFF, pitch_env: POFF, master_gain_db: -8.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "8-Bit", name: "8-Bit Noise",
        osc1: o(NOI, 0.8, 0.0, 0, true, 1, 0.0),
        osc2: OOFF,
        filter1: f(LP, 4000.0, 0.0, 1.0, 0.0, true),
        filter2: FOFF,
        amp_env: e(0.001, 0.10, 0.0, 0.05),
        filter1_env: EOFF, filter2_env: EOFF, pitch_env: POFF, master_gain_db: -6.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "8-Bit", name: "8-Bit Triangle",
        osc1: o(TRI, 0.8, 0.0, 0, true, 1, 0.0),
        osc2: OOFF,
        filter1: FOFF,
        filter2: FOFF,
        amp_env: e(0.001, 0.05, 0.85, 0.05),
        filter1_env: EOFF, filter2_env: EOFF, pitch_env: POFF, master_gain_db: -5.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "8-Bit", name: "8-Bit Kick",
        osc1: o(SQR, 0.9, 0.0, -2, true, 1, 0.0),
        osc2: OOFF,
        filter1: FOFF,
        filter2: FOFF,
        amp_env: e(0.001, 0.12, 0.0, 0.06),
        filter1_env: EOFF, filter2_env: EOFF,
        pitch_env: pe(0.001, 0.04, 0.0, 0.02, 36.0),
        master_gain_db: -4.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "8-Bit", name: "8-Bit Snare",
        osc1: o(NOI, 0.7, 0.0, 0, true, 1, 0.0),
        osc2: o(SQR, 0.4, 0.0, 1, true, 1, 0.0),
        filter1: f(LP, 6000.0, 0.0, 1.0, 0.0, true),
        filter2: FOFF,
        amp_env: e(0.001, 0.08, 0.0, 0.04),
        filter1_env: EOFF, filter2_env: EOFF,
        pitch_env: pe(0.001, 0.02, 0.0, 0.01, 12.0),
        master_gain_db: -5.0,
        fx: FX_OFF, arp: ARP_OFF,
    },

    // ===================================================================
    // CRAZY PRESETS
    // ===================================================================
    Preset { category: "Fun", name: "Alf",
        // Alien sitcom creature: wobbly, nasal, slightly unhinged.
        osc1: o(SQR, 0.6, -15.0, 0, true, 5, 50.0),
        osc2: o(SAW, 0.5, 20.0, 1, true, 3, 35.0),
        filter1: f(BP, 1200.0, 0.60, 1.5, 0.70, true),
        filter2: f(NT, 2500.0, 0.50, 1.0, 0.30, true),
        amp_env: e(0.01, 0.2, 0.75, 0.2),
        filter1_env: e(0.05, 0.4, 0.40, 0.3),
        filter2_env: e(0.1, 0.6, 0.30, 0.4),
        pitch_env: pe(0.01, 0.15, 0.20, 0.1, 5.0),
        master_gain_db: -7.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Fun", name: "Cat",
        // Meow: sine with slow pitch envelope sweep downward.
        osc1: o(SIN, 0.8, 0.0, 1, true, 1, 0.0),
        osc2: o(NOI, 0.06, 0.0, 0, true, 1, 0.0),
        filter1: f(LP, 2500.0, 0.20, 1.0, 0.30, true),
        filter2: f(BP, 1800.0, 0.35, 1.0, 0.0, true),
        amp_env: e(0.05, 0.3, 0.50, 0.25),
        filter1_env: e(0.05, 0.25, 0.25, 0.2),
        filter2_env: EOFF,
        pitch_env: pe(0.02, 0.20, 0.0, 0.15, -8.0),
        master_gain_db: -6.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Fun", name: "Elton",
        // The Rocket Man: flashy, bright, over-the-top piano with all the glitter.
        osc1: o(SAW, 0.8, -6.0, 0, true, 7, 18.0),
        osc2: o(SAW, 0.6, 6.0, -1, true, 5, 15.0),
        filter1: f(LP, 5000.0, 0.30, 1.3, 0.80, true),
        filter2: f(LP, 8000.0, 0.10, 1.0, 0.20, true),
        amp_env: e(0.002, 1.5, 0.15, 0.5),
        filter1_env: e(0.002, 0.5, 0.0, 0.3),
        filter2_env: e(0.002, 0.3, 0.10, 0.2),
        pitch_env: POFF,
        master_gain_db: -7.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
    Preset { category: "Fun", name: "Grand Pa",
        // "The most tremendous fart, believe me. Nobody farts better."
        osc1: o(NOI, 0.6, 0.0, 0, true, 1, 0.0),
        osc2: o(SAW, 0.7, 0.0, -2, true, 3, 40.0),
        filter1: f(LP, 400.0, 0.50, 3.0, 0.40, true),
        filter2: f(BP, 200.0, 0.60, 2.0, 0.20, true),
        amp_env: e(0.02, 0.5, 0.30, 0.4),
        filter1_env: e(0.01, 0.4, 0.20, 0.3),
        filter2_env: e(0.02, 0.5, 0.15, 0.4),
        pitch_env: pe(0.01, 0.3, 0.0, 0.2, -18.0),
        master_gain_db: -5.0,
        fx: FX_OFF, arp: ARP_OFF,
    },
];

/// Ordered list of preset categories.
pub const CATEGORIES: &[&str] = &[
    "Common", "Bass", "Keys", "Pads", "Drums", "Oneshots",
    "Arpeggios", "Soundscapes", "Atmospheres", "8-Bit", "Fun",
];

/// Return all presets belonging to a given category.
pub fn presets_in_category(category: &str) -> Vec<&'static Preset> {
    FACTORY_PRESETS.iter().filter(|p| p.category == category).collect()
}

// -----------------------------------------------------------------------
// Preset application helpers
// -----------------------------------------------------------------------

pub fn apply_preset(preset: &Preset, params: &Arc<SynthParams>, ctx: &dyn GuiContext) {
    apply_osc(&preset.osc1, &params.osc1, ctx);
    apply_osc(&preset.osc2, &params.osc2, ctx);
    apply_filter(&preset.filter1, &params.filter1, ctx);
    apply_filter(&preset.filter2, &params.filter2, ctx);
    apply_envelope(&preset.amp_env, &params.amp_env, ctx);
    apply_envelope(&preset.filter1_env, &params.filter1_env, ctx);
    apply_envelope(&preset.filter2_env, &params.filter2_env, ctx);
    apply_pitch_env(&preset.pitch_env, &params.pitch_env, ctx);
    set_float(ctx, &params.master_gain, util::db_to_gain(preset.master_gain_db));
    apply_fx(&preset.fx, params, ctx);
    apply_arp(&preset.arp, params, ctx);
}

fn apply_fx(src: &FxPreset, params: &Arc<SynthParams>, ctx: &dyn GuiContext) {
    set_float(ctx, &params.chorus.rate, src.chorus.rate);
    set_float(ctx, &params.chorus.depth, src.chorus.depth);
    set_float(ctx, &params.chorus.mix, src.chorus.mix);
    set_bool(ctx, &params.chorus.enabled, src.chorus.enabled);

    set_float(ctx, &params.delay.time_ms, src.delay.time_ms);
    set_float(ctx, &params.delay.feedback, src.delay.feedback);
    set_float(ctx, &params.delay.tone, src.delay.tone);
    set_float(ctx, &params.delay.mix, src.delay.mix);
    set_bool(ctx, &params.delay.enabled, src.delay.enabled);

    set_float(ctx, &params.shimmer.time_ms, src.shimmer.time_ms);
    set_float(ctx, &params.shimmer.feedback, src.shimmer.feedback);
    set_float(ctx, &params.shimmer.mix, src.shimmer.mix);
    set_bool(ctx, &params.shimmer.enabled, src.shimmer.enabled);

    set_enum(ctx, &params.gapper.rate, src.gapper.rate);
    set_float(ctx, &params.gapper.duty, src.gapper.duty);
    set_float(ctx, &params.gapper.smooth, src.gapper.smooth);
    set_float(ctx, &params.gapper.depth, src.gapper.depth);
    set_bool(ctx, &params.gapper.enabled, src.gapper.enabled);
}

fn apply_arp(src: &ArpPreset, params: &Arc<SynthParams>, ctx: &dyn GuiContext) {
    set_enum(ctx, &params.arp.pattern, src.pattern);
    set_enum(ctx, &params.arp.rate, src.rate);
    set_int(ctx, &params.arp.octaves, src.octaves);
    set_float(ctx, &params.arp.gate, src.gate);
    set_bool(ctx, &params.arp.enabled, src.enabled);
}

fn apply_osc(src: &OscPreset, dst: &crate::params::OscParams, ctx: &dyn GuiContext) {
    set_enum(ctx, &dst.waveform, src.waveform);
    set_float(ctx, &dst.level, src.level);
    set_float(ctx, &dst.detune, src.detune);
    set_int(ctx, &dst.octave, src.octave);
    set_bool(ctx, &dst.enabled, src.enabled);
    set_int(ctx, &dst.unison_voices, src.unison_voices);
    set_float(ctx, &dst.unison_spread, src.unison_spread);
    set_float(ctx, &dst.pan, src.pan);
    set_float(ctx, &dst.stereo_spread, src.stereo_spread);
}

fn apply_filter(src: &FilterPreset, dst: &crate::params::FilterParams, ctx: &dyn GuiContext) {
    set_enum(ctx, &dst.filter_type, src.filter_type);
    set_float(ctx, &dst.cutoff, src.cutoff);
    set_float(ctx, &dst.resonance, src.resonance);
    set_float(ctx, &dst.drive, src.drive);
    set_float(ctx, &dst.env_amount, src.env_amount);
    set_bool(ctx, &dst.enabled, src.enabled);
}

fn apply_envelope(src: &EnvPreset, dst: &crate::params::EnvelopeParams, ctx: &dyn GuiContext) {
    set_float(ctx, &dst.attack, src.attack);
    set_float(ctx, &dst.decay, src.decay);
    set_float(ctx, &dst.sustain, src.sustain);
    set_float(ctx, &dst.release, src.release);
}

fn apply_pitch_env(src: &PitchEnvPreset, dst: &crate::params::PitchEnvParams, ctx: &dyn GuiContext) {
    set_float(ctx, &dst.attack, src.attack);
    set_float(ctx, &dst.decay, src.decay);
    set_float(ctx, &dst.sustain, src.sustain);
    set_float(ctx, &dst.release, src.release);
    set_float(ctx, &dst.amount, src.amount);
}

fn set_float(ctx: &dyn GuiContext, param: &FloatParam, value: f32) {
    let normalized = param.preview_normalized(value);
    unsafe {
        ctx.raw_begin_set_parameter(param.as_ptr());
        ctx.raw_set_parameter_normalized(param.as_ptr(), normalized);
        ctx.raw_end_set_parameter(param.as_ptr());
    }
}

fn set_int(ctx: &dyn GuiContext, param: &IntParam, value: i32) {
    let normalized = param.preview_normalized(value);
    unsafe {
        ctx.raw_begin_set_parameter(param.as_ptr());
        ctx.raw_set_parameter_normalized(param.as_ptr(), normalized);
        ctx.raw_end_set_parameter(param.as_ptr());
    }
}

fn set_bool(ctx: &dyn GuiContext, param: &BoolParam, value: bool) {
    let normalized = if value { 1.0 } else { 0.0 };
    unsafe {
        ctx.raw_begin_set_parameter(param.as_ptr());
        ctx.raw_set_parameter_normalized(param.as_ptr(), normalized);
        ctx.raw_end_set_parameter(param.as_ptr());
    }
}

fn set_enum<E: Enum + PartialEq>(ctx: &dyn GuiContext, param: &EnumParam<E>, value: E) {
    let normalized = param.preview_normalized(value);
    unsafe {
        ctx.raw_begin_set_parameter(param.as_ptr());
        ctx.raw_set_parameter_normalized(param.as_ptr(), normalized);
        ctx.raw_end_set_parameter(param.as_ptr());
    }
}
