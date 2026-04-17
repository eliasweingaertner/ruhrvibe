//! Factory presets.
//!
//! Presets are defined as compile-time constant structs containing concrete
//! parameter values. On selection, `apply_preset` writes each value through
//! the `GuiContext` so DAW automation is notified of the changes.

use nih_plug::prelude::*;
use std::sync::Arc;

use crate::params::{FilterType, SynthParams, Waveform};

/// Snapshot of oscillator values for a preset.
#[derive(Clone, Copy)]
pub struct OscPreset {
    pub waveform: Waveform,
    pub level: f32,
    pub detune: f32,
    pub octave: i32,
    pub enabled: bool,
    pub unison_voices: i32,
    pub unison_spread: f32,
}

/// Snapshot of filter values for a preset.
#[derive(Clone, Copy)]
pub struct FilterPreset {
    pub filter_type: FilterType,
    pub cutoff: f32,
    pub resonance: f32,
    pub drive: f32,
    pub env_amount: f32,
    pub enabled: bool,
}

/// Snapshot of envelope values for a preset.
#[derive(Clone, Copy)]
pub struct EnvPreset {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

/// Snapshot of pitch envelope values.
#[derive(Clone, Copy)]
pub struct PitchEnvPreset {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub amount: f32,
}

/// Complete preset.
#[derive(Clone, Copy)]
pub struct Preset {
    pub name: &'static str,
    pub osc1: OscPreset,
    pub osc2: OscPreset,
    pub filter1: FilterPreset,
    pub filter2: FilterPreset,
    pub amp_env: EnvPreset,
    pub filter1_env: EnvPreset,
    pub filter2_env: EnvPreset,
    pub pitch_env: PitchEnvPreset,
    pub master_gain_db: f32,
}

const fn osc(
    waveform: Waveform,
    level: f32,
    detune: f32,
    octave: i32,
    enabled: bool,
    unison_voices: i32,
    unison_spread: f32,
) -> OscPreset {
    OscPreset { waveform, level, detune, octave, enabled, unison_voices, unison_spread }
}

const fn flt(
    filter_type: FilterType,
    cutoff: f32,
    resonance: f32,
    drive: f32,
    env_amount: f32,
    enabled: bool,
) -> FilterPreset {
    FilterPreset { filter_type, cutoff, resonance, drive, env_amount, enabled }
}

const fn env(attack: f32, decay: f32, sustain: f32, release: f32) -> EnvPreset {
    EnvPreset { attack, decay, sustain, release }
}

const fn penv(attack: f32, decay: f32, sustain: f32, release: f32, amount: f32) -> PitchEnvPreset {
    PitchEnvPreset { attack, decay, sustain, release, amount }
}

/// No pitch envelope modulation.
const PENV_OFF: PitchEnvPreset = penv(0.001, 0.1, 0.0, 0.05, 0.0);

pub const FACTORY_PRESETS: &[Preset] = &[
    // ---------------------------------------------------------------
    // Melodic presets
    // ---------------------------------------------------------------
    Preset {
        name: "Init",
        osc1: osc(Waveform::Saw, 0.75, 0.0, 0, true, 1, 20.0),
        osc2: osc(Waveform::Square, 0.0, 0.0, 0, false, 1, 20.0),
        filter1: flt(FilterType::LowPass, 12_000.0, 0.2, 1.0, 0.0, true),
        filter2: flt(FilterType::LowPass, 12_000.0, 0.0, 1.0, 0.0, false),
        amp_env: env(0.01, 0.3, 0.7, 0.3),
        filter1_env: env(0.01, 0.3, 0.5, 0.3),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        pitch_env: PENV_OFF,
        master_gain_db: -6.0,
    },
    Preset {
        name: "Fat Bass",
        osc1: osc(Waveform::Saw, 0.8, -7.0, -1, true, 3, 12.0),
        osc2: osc(Waveform::Saw, 0.7, 7.0, -1, true, 3, 12.0),
        filter1: flt(FilterType::LowPass, 500.0, 0.6, 1.5, 0.6, true),
        filter2: flt(FilterType::LowPass, 12_000.0, 0.0, 1.0, 0.0, false),
        amp_env: env(0.005, 0.2, 0.8, 0.15),
        filter1_env: env(0.005, 0.25, 0.3, 0.2),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        pitch_env: PENV_OFF,
        master_gain_db: -6.0,
    },
    Preset {
        name: "Warm Pad",
        osc1: osc(Waveform::Triangle, 0.7, 0.0, 0, true, 5, 25.0),
        osc2: osc(Waveform::Saw, 0.5, 5.0, 0, true, 5, 30.0),
        filter1: flt(FilterType::LowPass, 2000.0, 0.2, 1.0, 0.3, true),
        filter2: flt(FilterType::LowPass, 12_000.0, 0.0, 1.0, 0.0, false),
        amp_env: env(1.2, 0.5, 0.8, 1.8),
        filter1_env: env(1.5, 1.5, 0.5, 1.5),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        pitch_env: PENV_OFF,
        master_gain_db: -8.0,
    },
    Preset {
        name: "Pluck",
        osc1: osc(Waveform::Square, 0.75, 0.0, 0, true, 1, 20.0),
        osc2: osc(Waveform::Square, 0.0, 0.0, 0, false, 1, 20.0),
        filter1: flt(FilterType::LowPass, 3000.0, 0.4, 1.2, 0.8, true),
        filter2: flt(FilterType::LowPass, 12_000.0, 0.0, 1.0, 0.0, false),
        amp_env: env(0.003, 0.25, 0.0, 0.25),
        filter1_env: env(0.003, 0.2, 0.0, 0.2),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        pitch_env: PENV_OFF,
        master_gain_db: -6.0,
    },
    Preset {
        name: "Lead",
        osc1: osc(Waveform::Saw, 0.7, -3.0, 0, true, 3, 15.0),
        osc2: osc(Waveform::Square, 0.5, 3.0, 0, true, 1, 20.0),
        filter1: flt(FilterType::LowPass, 4500.0, 0.35, 1.3, 0.3, true),
        filter2: flt(FilterType::LowPass, 12_000.0, 0.0, 1.0, 0.0, false),
        amp_env: env(0.02, 0.2, 0.75, 0.2),
        filter1_env: env(0.05, 0.3, 0.6, 0.3),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        pitch_env: PENV_OFF,
        master_gain_db: -6.0,
    },
    Preset {
        name: "Sub Bass",
        osc1: osc(Waveform::Sine, 0.9, 0.0, -2, true, 1, 0.0),
        osc2: osc(Waveform::Sine, 0.0, 0.0, 0, false, 1, 0.0),
        filter1: flt(FilterType::LowPass, 12_000.0, 0.0, 1.0, 0.0, false),
        filter2: flt(FilterType::LowPass, 12_000.0, 0.0, 1.0, 0.0, false),
        amp_env: env(0.01, 0.1, 0.9, 0.1),
        filter1_env: env(0.01, 0.3, 0.5, 0.3),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        pitch_env: PENV_OFF,
        master_gain_db: -4.0,
    },
    Preset {
        name: "Strings",
        osc1: osc(Waveform::Triangle, 0.7, -5.0, 0, true, 5, 18.0),
        osc2: osc(Waveform::Triangle, 0.7, 5.0, 0, true, 5, 18.0),
        filter1: flt(FilterType::LowPass, 5000.0, 0.15, 1.0, 0.2, true),
        filter2: flt(FilterType::LowPass, 12_000.0, 0.0, 1.0, 0.0, false),
        amp_env: env(0.8, 0.4, 0.85, 1.2),
        filter1_env: env(1.2, 0.5, 0.7, 1.2),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        pitch_env: PENV_OFF,
        master_gain_db: -8.0,
    },
    Preset {
        name: "Brass",
        osc1: osc(Waveform::Saw, 0.8, 0.0, 0, true, 3, 10.0),
        osc2: osc(Waveform::Saw, 0.4, 8.0, 0, true, 1, 20.0),
        filter1: flt(FilterType::LowPass, 1500.0, 0.3, 1.2, 0.7, true),
        filter2: flt(FilterType::LowPass, 12_000.0, 0.0, 1.0, 0.0, false),
        amp_env: env(0.08, 0.3, 0.7, 0.3),
        filter1_env: env(0.05, 0.5, 0.4, 0.3),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        pitch_env: PENV_OFF,
        master_gain_db: -6.0,
    },
    Preset {
        name: "Dreamy Pan Flute",
        osc1: osc(Waveform::Sine, 0.8, 0.0, 0, true, 3, 8.0),
        osc2: osc(Waveform::Triangle, 0.25, 3.0, 0, true, 3, 10.0),
        filter1: flt(FilterType::LowPass, 3000.0, 0.25, 1.0, 0.15, true),
        filter2: flt(FilterType::Notch, 1800.0, 0.4, 1.0, 0.0, true),
        amp_env: env(0.08, 0.2, 0.7, 0.4),
        filter1_env: env(0.1, 0.5, 0.8, 0.3),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        pitch_env: PENV_OFF,
        master_gain_db: -7.0,
    },
    Preset {
        name: "Piano",
        osc1: osc(Waveform::Saw, 0.75, -4.0, 0, true, 2, 6.0),
        osc2: osc(Waveform::Saw, 0.55, 4.0, -1, true, 2, 6.0),
        filter1: flt(FilterType::LowPass, 1800.0, 0.2, 1.0, 0.85, true),
        filter2: flt(FilterType::LowPass, 12_000.0, 0.0, 1.0, 0.0, false),
        amp_env: env(0.003, 0.9, 0.15, 0.35),
        filter1_env: env(0.003, 0.35, 0.0, 0.25),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        pitch_env: PENV_OFF,
        master_gain_db: -6.0,
    },
    Preset {
        name: "Swoosh",
        osc1: osc(Waveform::Square, 0.6, 12.0, 0, true, 5, 40.0),
        osc2: osc(Waveform::Saw, 0.6, -12.0, 0, true, 5, 40.0),
        filter1: flt(FilterType::BandPass, 800.0, 0.7, 1.0, 1.0, true),
        filter2: flt(FilterType::HighPass, 300.0, 0.5, 1.0, 0.9, true),
        amp_env: env(1.8, 0.5, 0.7, 2.5),
        filter1_env: env(2.0, 0.5, 0.8, 2.2),
        filter2_env: env(2.5, 0.3, 0.9, 2.0),
        pitch_env: PENV_OFF,
        master_gain_db: -8.0,
    },
    // ---------------------------------------------------------------
    // Supersaw / rich textures
    // ---------------------------------------------------------------
    Preset {
        name: "Supersaw",
        osc1: osc(Waveform::Saw, 0.8, 0.0, 0, true, 7, 35.0),
        osc2: osc(Waveform::Saw, 0.6, 0.0, 1, true, 7, 40.0),
        filter1: flt(FilterType::LowPass, 8000.0, 0.15, 1.0, 0.2, true),
        filter2: flt(FilterType::LowPass, 12_000.0, 0.0, 1.0, 0.0, false),
        amp_env: env(0.02, 0.3, 0.8, 0.5),
        filter1_env: env(0.01, 0.5, 0.6, 0.4),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        pitch_env: PENV_OFF,
        master_gain_db: -8.0,
    },
    // ---------------------------------------------------------------
    // Drum presets
    // ---------------------------------------------------------------
    Preset {
        name: "Kick",
        // Sine with fast pitch sweep from high to low.
        osc1: osc(Waveform::Sine, 0.95, 0.0, -1, true, 1, 0.0),
        osc2: osc(Waveform::Sine, 0.0, 0.0, 0, false, 1, 0.0),
        filter1: flt(FilterType::LowPass, 500.0, 0.0, 1.0, 0.0, true),
        filter2: flt(FilterType::LowPass, 12_000.0, 0.0, 1.0, 0.0, false),
        // Very fast amp: instant attack, short decay, no sustain.
        amp_env: env(0.001, 0.25, 0.0, 0.15),
        filter1_env: env(0.01, 0.3, 0.5, 0.3),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        // Pitch drops ~36 semitones very fast.
        pitch_env: penv(0.001, 0.06, 0.0, 0.05, 36.0),
        master_gain_db: -3.0,
    },
    Preset {
        name: "Snare",
        // Sine body + noise for rattle.
        osc1: osc(Waveform::Sine, 0.7, 0.0, 0, true, 1, 0.0),
        osc2: osc(Waveform::Noise, 0.8, 0.0, 0, true, 1, 0.0),
        filter1: flt(FilterType::BandPass, 2000.0, 0.3, 1.2, 0.0, true),
        filter2: flt(FilterType::HighPass, 500.0, 0.1, 1.0, 0.0, true),
        amp_env: env(0.001, 0.15, 0.0, 0.12),
        filter1_env: env(0.01, 0.3, 0.5, 0.3),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        // Slight pitch drop for the tonal body.
        pitch_env: penv(0.001, 0.04, 0.0, 0.03, 12.0),
        master_gain_db: -4.0,
    },
    Preset {
        name: "Hi-Hat",
        // Pure noise through a high-pass filter, very short.
        osc1: osc(Waveform::Noise, 0.9, 0.0, 0, true, 1, 0.0),
        osc2: osc(Waveform::Square, 0.15, 0.0, 2, true, 1, 0.0),
        filter1: flt(FilterType::HighPass, 7000.0, 0.3, 1.0, 0.0, true),
        filter2: flt(FilterType::BandPass, 10000.0, 0.5, 1.0, 0.0, true),
        amp_env: env(0.001, 0.06, 0.0, 0.04),
        filter1_env: env(0.01, 0.3, 0.5, 0.3),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        pitch_env: PENV_OFF,
        master_gain_db: -6.0,
    },
    Preset {
        name: "Tom",
        // Sine body with pitch sweep, longer than kick.
        osc1: osc(Waveform::Sine, 0.9, 0.0, 0, true, 1, 0.0),
        osc2: osc(Waveform::Noise, 0.2, 0.0, 0, true, 1, 0.0),
        filter1: flt(FilterType::LowPass, 1500.0, 0.2, 1.0, 0.0, true),
        filter2: flt(FilterType::LowPass, 12_000.0, 0.0, 1.0, 0.0, false),
        amp_env: env(0.001, 0.35, 0.0, 0.2),
        filter1_env: env(0.01, 0.3, 0.5, 0.3),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        pitch_env: penv(0.001, 0.08, 0.0, 0.05, 24.0),
        master_gain_db: -4.0,
    },
    Preset {
        name: "Clap",
        // Noise burst, double-hit via slightly longer attack.
        osc1: osc(Waveform::Noise, 0.95, 0.0, 0, true, 1, 0.0),
        osc2: osc(Waveform::Noise, 0.0, 0.0, 0, false, 1, 0.0),
        filter1: flt(FilterType::BandPass, 1200.0, 0.5, 1.3, 0.0, true),
        filter2: flt(FilterType::HighPass, 600.0, 0.2, 1.0, 0.0, true),
        amp_env: env(0.001, 0.18, 0.0, 0.15),
        filter1_env: env(0.01, 0.3, 0.5, 0.3),
        filter2_env: env(0.01, 0.3, 0.5, 0.3),
        pitch_env: PENV_OFF,
        master_gain_db: -5.0,
    },
];

/// Apply a preset by pushing each parameter value through the GuiContext.
/// This notifies the host of automatable parameter changes.
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
}

fn apply_osc(src: &OscPreset, dst: &crate::params::OscParams, ctx: &dyn GuiContext) {
    set_enum(ctx, &dst.waveform, src.waveform);
    set_float(ctx, &dst.level, src.level);
    set_float(ctx, &dst.detune, src.detune);
    set_int(ctx, &dst.octave, src.octave);
    set_bool(ctx, &dst.enabled, src.enabled);
    set_int(ctx, &dst.unison_voices, src.unison_voices);
    set_float(ctx, &dst.unison_spread, src.unison_spread);
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
