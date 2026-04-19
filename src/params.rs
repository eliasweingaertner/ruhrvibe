//! Parameter definitions for the subtractive synth.
//!
//! Uses nih-plug's `Params` derive macro. Parameters are grouped into nested
//! structs for organization: oscillators, filters, envelopes, master.

use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

/// Oscillator waveform selection.
#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Waveform {
    #[id = "sine"]
    #[name = "Sine"]
    Sine,
    #[id = "saw"]
    #[name = "Saw"]
    Saw,
    #[id = "square"]
    #[name = "Square"]
    Square,
    #[id = "triangle"]
    #[name = "Triangle"]
    Triangle,
    #[id = "noise"]
    #[name = "Noise"]
    Noise,
}

/// Filter type selection.
#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum FilterType {
    #[id = "lowpass"]
    #[name = "Low Pass"]
    LowPass,
    #[id = "highpass"]
    #[name = "High Pass"]
    HighPass,
    #[id = "bandpass"]
    #[name = "Band Pass"]
    BandPass,
    #[id = "notch"]
    #[name = "Notch"]
    Notch,
}

/// Parameters for a single oscillator.
#[derive(Params)]
pub struct OscParams {
    #[id = "wave"]
    pub waveform: EnumParam<Waveform>,

    #[id = "level"]
    pub level: FloatParam,

    #[id = "detune"]
    pub detune: FloatParam,

    #[id = "octave"]
    pub octave: IntParam,

    #[id = "on"]
    pub enabled: BoolParam,

    #[id = "uni"]
    pub unison_voices: IntParam,

    #[id = "unisp"]
    pub unison_spread: FloatParam,

    #[id = "pan"]
    pub pan: FloatParam,

    #[id = "stspr"]
    pub stereo_spread: FloatParam,
}

impl OscParams {
    fn new(default_enabled: bool, default_waveform: Waveform) -> Self {
        Self {
            waveform: EnumParam::new("Waveform", default_waveform),
            level: FloatParam::new(
                "Level",
                0.75,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(10.0))
            .with_value_to_string(formatters::v2s_f32_percentage(1))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            detune: FloatParam::new(
                "Detune",
                0.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_unit(" ct")
            .with_step_size(0.1),
            octave: IntParam::new("Octave", 0, IntRange::Linear { min: -3, max: 3 })
                .with_unit(" oct"),
            enabled: BoolParam::new("Enabled", default_enabled),
            unison_voices: IntParam::new("Unison", 1, IntRange::Linear { min: 1, max: 7 }),
            unison_spread: FloatParam::new(
                "Spread",
                20.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_unit(" ct")
            .with_step_size(0.1),
            pan: FloatParam::new(
                "Pan",
                0.0,
                FloatRange::Linear { min: -1.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_value_to_string(formatters::v2s_f32_panning())
            .with_string_to_value(formatters::s2v_f32_panning()),
            stereo_spread: FloatParam::new(
                "Stereo",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}

/// Parameters for a single filter slot.
#[derive(Params)]
pub struct FilterParams {
    #[id = "type"]
    pub filter_type: EnumParam<FilterType>,

    #[id = "cutoff"]
    pub cutoff: FloatParam,

    #[id = "res"]
    pub resonance: FloatParam,

    #[id = "drive"]
    pub drive: FloatParam,

    #[id = "envamt"]
    pub env_amount: FloatParam,

    #[id = "on"]
    pub enabled: BoolParam,
}

impl FilterParams {
    fn new(default_enabled: bool, default_cutoff: f32) -> Self {
        Self {
            filter_type: EnumParam::new("Type", FilterType::LowPass),
            cutoff: FloatParam::new(
                "Cutoff",
                default_cutoff,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20_000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(10.0))
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(1))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            resonance: FloatParam::new(
                "Resonance",
                0.3,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_value_to_string(formatters::v2s_f32_percentage(1))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            drive: FloatParam::new(
                "Drive",
                1.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 4.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_value_to_string(Arc::new(|v| format!("{:.2}x", v))),
            env_amount: FloatParam::new(
                "Env Amount",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_value_to_string(formatters::v2s_f32_percentage(1))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            enabled: BoolParam::new("Enabled", default_enabled),
        }
    }
}

/// ADSR envelope parameters.
#[derive(Params)]
pub struct EnvelopeParams {
    #[id = "a"]
    pub attack: FloatParam,

    #[id = "d"]
    pub decay: FloatParam,

    #[id = "s"]
    pub sustain: FloatParam,

    #[id = "r"]
    pub release: FloatParam,
}

impl EnvelopeParams {
    fn new(default_attack: f32, default_decay: f32, default_sustain: f32, default_release: f32) -> Self {
        let time_range = FloatRange::Skewed {
            min: 0.001,
            max: 10.0,
            factor: FloatRange::skew_factor(-2.0),
        };
        Self {
            attack: FloatParam::new("Attack", default_attack, time_range)
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_unit(" s")
                .with_value_to_string(Arc::new(|v| {
                    if v < 1.0 {
                        format!("{:.0} ms", v * 1000.0)
                    } else {
                        format!("{:.2} s", v)
                    }
                })),
            decay: FloatParam::new("Decay", default_decay, time_range)
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_unit(" s")
                .with_value_to_string(Arc::new(|v| {
                    if v < 1.0 {
                        format!("{:.0} ms", v * 1000.0)
                    } else {
                        format!("{:.2} s", v)
                    }
                })),
            sustain: FloatParam::new(
                "Sustain",
                default_sustain,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_value_to_string(formatters::v2s_f32_percentage(1))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            release: FloatParam::new("Release", default_release, time_range)
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_unit(" s")
                .with_value_to_string(Arc::new(|v| {
                    if v < 1.0 {
                        format!("{:.0} ms", v * 1000.0)
                    } else {
                        format!("{:.2} s", v)
                    }
                })),
        }
    }
}

/// Pitch envelope parameters: an ADSR that modulates pitch by a given amount.
#[derive(Params)]
pub struct PitchEnvParams {
    #[id = "a"]
    pub attack: FloatParam,

    #[id = "d"]
    pub decay: FloatParam,

    #[id = "s"]
    pub sustain: FloatParam,

    #[id = "r"]
    pub release: FloatParam,

    /// How many semitones the envelope sweeps (positive = up, negative = down).
    #[id = "amt"]
    pub amount: FloatParam,
}

impl PitchEnvParams {
    fn new() -> Self {
        let time_range = FloatRange::Skewed {
            min: 0.001,
            max: 10.0,
            factor: FloatRange::skew_factor(-2.0),
        };
        Self {
            attack: FloatParam::new("Attack", 0.001, time_range)
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_unit(" s")
                .with_value_to_string(Arc::new(|v| {
                    if v < 1.0 {
                        format!("{:.0} ms", v * 1000.0)
                    } else {
                        format!("{:.2} s", v)
                    }
                })),
            decay: FloatParam::new("Decay", 0.1, time_range)
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_unit(" s")
                .with_value_to_string(Arc::new(|v| {
                    if v < 1.0 {
                        format!("{:.0} ms", v * 1000.0)
                    } else {
                        format!("{:.2} s", v)
                    }
                })),
            sustain: FloatParam::new(
                "Sustain",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_value_to_string(formatters::v2s_f32_percentage(1))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            release: FloatParam::new("Release", 0.05, time_range)
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_unit(" s")
                .with_value_to_string(Arc::new(|v| {
                    if v < 1.0 {
                        format!("{:.0} ms", v * 1000.0)
                    } else {
                        format!("{:.2} s", v)
                    }
                })),
            amount: FloatParam::new(
                "Amount",
                0.0,
                FloatRange::Linear {
                    min: -48.0,
                    max: 48.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_unit(" st")
            .with_step_size(0.1),
        }
    }
}

/// Parameters for the stereo chorus effect.
#[derive(Params)]
pub struct ChorusFxParams {
    #[id = "rate"]
    pub rate: FloatParam,
    #[id = "depth"]
    pub depth: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
    #[id = "on"]
    pub enabled: BoolParam,
}

impl ChorusFxParams {
    fn new() -> Self {
        Self {
            rate: FloatParam::new(
                "Rate",
                0.5,
                FloatRange::Skewed {
                    min: 0.05,
                    max: 10.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(20.0))
            .with_unit(" Hz")
            .with_value_to_string(Arc::new(|v| format!("{:.2} Hz", v))),
            depth: FloatParam::new("Depth", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            mix: FloatParam::new("Mix", 0.35, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            enabled: BoolParam::new("Enabled", false),
        }
    }
}

/// Parameters for the stereo ping-pong delay.
#[derive(Params)]
pub struct DelayFxParams {
    #[id = "time"]
    pub time_ms: FloatParam,
    #[id = "fb"]
    pub feedback: FloatParam,
    #[id = "tone"]
    pub tone: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
    #[id = "on"]
    pub enabled: BoolParam,
}

impl DelayFxParams {
    fn new() -> Self {
        Self {
            time_ms: FloatParam::new(
                "Time",
                350.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 2000.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(30.0))
            .with_unit(" ms")
            .with_value_to_string(Arc::new(|v| format!("{:.0} ms", v))),
            feedback: FloatParam::new(
                "Feedback",
                0.35,
                FloatRange::Linear { min: 0.0, max: 0.95 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            tone: FloatParam::new(
                "Tone",
                0.6,
                FloatRange::Linear { min: 0.05, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            mix: FloatParam::new("Mix", 0.25, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            enabled: BoolParam::new("Enabled", false),
        }
    }
}

/// Top-level plugin parameters.
#[derive(Params)]
pub struct SynthParams {
    /// GUI window state — persisted across sessions.
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    #[nested(id_prefix = "osc1", group = "Oscillator 1")]
    pub osc1: Arc<OscParams>,

    #[nested(id_prefix = "osc2", group = "Oscillator 2")]
    pub osc2: Arc<OscParams>,

    #[nested(id_prefix = "flt1", group = "Filter 1")]
    pub filter1: Arc<FilterParams>,

    #[nested(id_prefix = "flt2", group = "Filter 2")]
    pub filter2: Arc<FilterParams>,

    #[nested(id_prefix = "ampenv", group = "Amp Envelope")]
    pub amp_env: Arc<EnvelopeParams>,

    #[nested(id_prefix = "flt1env", group = "Filter 1 Envelope")]
    pub filter1_env: Arc<EnvelopeParams>,

    #[nested(id_prefix = "flt2env", group = "Filter 2 Envelope")]
    pub filter2_env: Arc<EnvelopeParams>,

    #[nested(id_prefix = "pitchenv", group = "Pitch Envelope")]
    pub pitch_env: Arc<PitchEnvParams>,

    #[nested(id_prefix = "chorus", group = "Chorus")]
    pub chorus: Arc<ChorusFxParams>,

    #[nested(id_prefix = "delay", group = "Delay")]
    pub delay: Arc<DelayFxParams>,

    #[id = "master_gain"]
    pub master_gain: FloatParam,

    #[id = "num_voices"]
    pub num_voices: IntParam,
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            editor_state: crate::editor::default_state(),
            osc1: Arc::new(OscParams::new(true, Waveform::Saw)),
            osc2: Arc::new(OscParams::new(false, Waveform::Square)),
            filter1: Arc::new(FilterParams::new(true, 8000.0)),
            filter2: Arc::new(FilterParams::new(false, 12000.0)),
            amp_env: Arc::new(EnvelopeParams::new(0.01, 0.3, 0.7, 0.3)),
            filter1_env: Arc::new(EnvelopeParams::new(0.01, 0.5, 0.5, 0.3)),
            filter2_env: Arc::new(EnvelopeParams::new(0.01, 0.5, 0.5, 0.3)),
            pitch_env: Arc::new(PitchEnvParams::new()),
            chorus: Arc::new(ChorusFxParams::new()),
            delay: Arc::new(DelayFxParams::new()),
            master_gain: FloatParam::new(
                "Master Gain",
                util::db_to_gain(-6.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-60.0),
                    max: util::db_to_gain(6.0),
                    factor: FloatRange::gain_skew_factor(-60.0, 6.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(30.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            num_voices: IntParam::new(
                "Voices",
                16,
                IntRange::Linear { min: 1, max: 32 },
            ),
        }
    }
}
