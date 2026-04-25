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

/// Distortion waveshaper character.
#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum DistType {
    #[id = "soft"] #[name = "Soft"] Soft,
    #[id = "hard"] #[name = "Hard"] Hard,
    #[id = "fuzz"] #[name = "Fuzz"] Fuzz,
    #[id = "warm"] #[name = "Warm"] Warm,
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

    #[id = "fm"]
    pub fm_amount: FloatParam,
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
            fm_amount: FloatParam::new(
                "FM",
                0.0,
                FloatRange::Linear { min: 0.0, max: 5.0 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_step_size(0.01),
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

/// Host-synced musical rate shared by tempo-synced effects (Gapper, Arp).
#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum SyncRate {
    #[id = "1_1"]
    #[name = "1/1"]
    Whole,
    #[id = "1_2"]
    #[name = "1/2"]
    Half,
    #[id = "1_2d"]
    #[name = "1/2 D"]
    HalfDotted,
    #[id = "1_2t"]
    #[name = "1/2 T"]
    HalfTriplet,
    #[id = "1_4"]
    #[name = "1/4"]
    Quarter,
    #[id = "1_4d"]
    #[name = "1/4 D"]
    QuarterDotted,
    #[id = "1_4t"]
    #[name = "1/4 T"]
    QuarterTriplet,
    #[id = "1_8"]
    #[name = "1/8"]
    Eighth,
    #[id = "1_8d"]
    #[name = "1/8 D"]
    EighthDotted,
    #[id = "1_8t"]
    #[name = "1/8 T"]
    EighthTriplet,
    #[id = "1_16"]
    #[name = "1/16"]
    Sixteenth,
    #[id = "1_16t"]
    #[name = "1/16 T"]
    SixteenthTriplet,
    #[id = "1_32"]
    #[name = "1/32"]
    ThirtySecond,
}

impl SyncRate {
    /// Length of one gate cycle in beats (1 beat = 1 quarter note).
    pub fn beats_per_cycle(self) -> f32 {
        match self {
            SyncRate::Whole => 4.0,
            SyncRate::Half => 2.0,
            SyncRate::HalfDotted => 3.0,
            SyncRate::HalfTriplet => 4.0 / 3.0,
            SyncRate::Quarter => 1.0,
            SyncRate::QuarterDotted => 1.5,
            SyncRate::QuarterTriplet => 2.0 / 3.0,
            SyncRate::Eighth => 0.5,
            SyncRate::EighthDotted => 0.75,
            SyncRate::EighthTriplet => 1.0 / 3.0,
            SyncRate::Sixteenth => 0.25,
            SyncRate::SixteenthTriplet => 1.0 / 6.0,
            SyncRate::ThirtySecond => 0.125,
        }
    }
}

/// Musical scale for arpeggiator pitch-snapping. `Off` keeps pitches chromatic.
#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ArpScale {
    #[id = "off"]
    #[name = "Off"]
    Off,
    #[id = "maj"]
    #[name = "Major"]
    Major,
    #[id = "min"]
    #[name = "Minor"]
    Minor,
    #[id = "pma"]
    #[name = "Penta Maj"]
    PentaMajor,
    #[id = "pmi"]
    #[name = "Penta Min"]
    PentaMinor,
    #[id = "dor"]
    #[name = "Dorian"]
    Dorian,
    #[id = "mix"]
    #[name = "Mixolydian"]
    Mixolydian,
    #[id = "blu"]
    #[name = "Blues"]
    Blues,
}

impl ArpScale {
    /// Bitmask of allowed pitch classes relative to the root (bit i = pc i).
    pub fn mask(self) -> u16 {
        match self {
            // 0, 1, 2, ..., 11
            ArpScale::Off => 0xFFF,
            // 0, 2, 4, 5, 7, 9, 11
            ArpScale::Major => 0b1010_1101_0101,
            // 0, 2, 3, 5, 7, 8, 10
            ArpScale::Minor => 0b0101_1010_1101,
            // 0, 2, 4, 7, 9
            ArpScale::PentaMajor => 0b0010_1001_0101,
            // 0, 3, 5, 7, 10
            ArpScale::PentaMinor => 0b0100_1010_1001,
            // 0, 2, 3, 5, 7, 9, 10
            ArpScale::Dorian => 0b0110_1010_1101,
            // 0, 2, 4, 5, 7, 9, 10
            ArpScale::Mixolydian => 0b0110_1011_0101,
            // 0, 3, 5, 6, 7, 10
            ArpScale::Blues => 0b0100_1111_1001,
        }
    }
}

/// Root pitch class for scale-locked arpeggiation.
#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ArpRoot {
    #[id = "c"]  #[name = "C"]  C,
    #[id = "cs"] #[name = "C#"] CSharp,
    #[id = "d"]  #[name = "D"]  D,
    #[id = "ds"] #[name = "D#"] DSharp,
    #[id = "e"]  #[name = "E"]  E,
    #[id = "f"]  #[name = "F"]  F,
    #[id = "fs"] #[name = "F#"] FSharp,
    #[id = "g"]  #[name = "G"]  G,
    #[id = "gs"] #[name = "G#"] GSharp,
    #[id = "a"]  #[name = "A"]  A,
    #[id = "as"] #[name = "A#"] ASharp,
    #[id = "b"]  #[name = "B"]  B,
}

impl ArpRoot {
    pub fn semitones(self) -> u8 {
        match self {
            ArpRoot::C => 0, ArpRoot::CSharp => 1, ArpRoot::D => 2, ArpRoot::DSharp => 3,
            ArpRoot::E => 4, ArpRoot::F => 5, ArpRoot::FSharp => 6, ArpRoot::G => 7,
            ArpRoot::GSharp => 8, ArpRoot::A => 9, ArpRoot::ASharp => 10, ArpRoot::B => 11,
        }
    }
}

/// Note-order pattern for the arpeggiator.
#[derive(Enum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ArpPattern {
    #[id = "up"]
    #[name = "Up"]
    Up,
    #[id = "down"]
    #[name = "Down"]
    Down,
    #[id = "updown"]
    #[name = "Up/Down"]
    UpDown,
    #[id = "random"]
    #[name = "Random"]
    Random,
    #[id = "asplayed"]
    #[name = "As Played"]
    AsPlayed,
}

/// Parameters for the host-synced arpeggiator.
#[derive(Params)]
pub struct ArpParams {
    #[id = "pat"]
    pub pattern: EnumParam<ArpPattern>,
    #[id = "rate"]
    pub rate: EnumParam<SyncRate>,
    #[id = "oct"]
    pub octaves: IntParam,
    #[id = "gate"]
    pub gate: FloatParam,
    #[id = "scale"]
    pub scale: EnumParam<ArpScale>,
    #[id = "root"]
    pub root: EnumParam<ArpRoot>,
    /// When on, one held note expands into a scale walk instead of
    /// cycling through held chord tones. Requires a non-Off scale.
    #[id = "walk"]
    pub walk: BoolParam,
    /// Scale-degree stride per arp tick (1 = seconds, 2 = thirds, 3 = fourths…).
    #[id = "step"]
    pub step: IntParam,
    #[id = "on"]
    pub enabled: BoolParam,
}

impl ArpParams {
    fn new() -> Self {
        Self {
            pattern: EnumParam::new("Pattern", ArpPattern::Up),
            rate: EnumParam::new("Rate", SyncRate::Sixteenth),
            octaves: IntParam::new("Octaves", 1, IntRange::Linear { min: 1, max: 4 })
                .with_unit(" oct"),
            gate: FloatParam::new(
                "Gate",
                0.5,
                FloatRange::Linear { min: 0.05, max: 0.95 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            scale: EnumParam::new("Scale", ArpScale::Off),
            root: EnumParam::new("Root", ArpRoot::C),
            walk: BoolParam::new("Walk", false),
            step: IntParam::new("Step", 1, IntRange::Linear { min: 1, max: 7 }),
            enabled: BoolParam::new("Enabled", false),
        }
    }
}

/// Parameters for the host-synced rhythmic gate.
#[derive(Params)]
pub struct GapperFxParams {
    #[id = "rate"]
    pub rate: EnumParam<SyncRate>,
    #[id = "duty"]
    pub duty: FloatParam,
    #[id = "smooth"]
    pub smooth: FloatParam,
    #[id = "depth"]
    pub depth: FloatParam,
    #[id = "on"]
    pub enabled: BoolParam,
}

impl GapperFxParams {
    fn new() -> Self {
        Self {
            rate: EnumParam::new("Rate", SyncRate::Eighth),
            duty: FloatParam::new("Duty", 0.5, FloatRange::Linear { min: 0.05, max: 0.95 })
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            smooth: FloatParam::new(
                "Smooth",
                0.1,
                FloatRange::Linear { min: 0.0, max: 0.5 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            depth: FloatParam::new(
                "Depth",
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(20.0))
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
            enabled: BoolParam::new("Enabled", false),
        }
    }
}

/// Parameters for the shimmer delay.
#[derive(Params)]
pub struct ShimmerFxParams {
    #[id = "time"]
    pub time_ms: FloatParam,
    #[id = "fb"]
    pub feedback: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
    #[id = "on"]
    pub enabled: BoolParam,
}

impl ShimmerFxParams {
    fn new() -> Self {
        Self {
            time_ms: FloatParam::new(
                "Time",
                500.0,
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
                0.45,
                FloatRange::Linear { min: 0.0, max: 0.9 },
            )
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

/// Parameters for the plate reverb.
#[derive(Params)]
pub struct ReverbFxParams {
    #[id = "on"]   pub enabled:   BoolParam,
    #[id = "size"] pub room_size: FloatParam,
    #[id = "damp"] pub damping:   FloatParam,
    #[id = "wid"]  pub width:     FloatParam,
    #[id = "mix"]  pub mix:       FloatParam,
}

impl ReverbFxParams {
    fn new() -> Self {
        Self {
            enabled:   BoolParam::new("Enabled", false),
            room_size: FloatParam::new("Size", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            damping:   FloatParam::new("Damping", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            width:     FloatParam::new("Width", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            mix:       FloatParam::new("Mix", 0.25, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}

/// Parameters for the waveshaping distortion.
#[derive(Params)]
pub struct DistortionFxParams {
    #[id = "on"]   pub enabled:   BoolParam,
    #[id = "drv"]  pub drive:     FloatParam,
    #[id = "type"] pub dist_type: EnumParam<DistType>,
    #[id = "tone"] pub tone:      FloatParam,
    #[id = "mix"]  pub mix:       FloatParam,
}

impl DistortionFxParams {
    fn new() -> Self {
        Self {
            enabled:   BoolParam::new("Enabled", false),
            drive:     FloatParam::new("Drive", 2.0,
                FloatRange::Skewed { min: 1.0, max: 20.0, factor: FloatRange::skew_factor(-1.5) })
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_value_to_string(Arc::new(|v| format!("{:.1}x", v))),
            dist_type: EnumParam::new("Type", DistType::Soft),
            tone:      FloatParam::new("Tone", 0.8, FloatRange::Linear { min: 0.05, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            mix:       FloatParam::new("Mix", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(20.0))
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
        }
    }
}

/// Per-effect position in the FX chain (1 = first, 6 = last).
/// The synth sorts effects by this value before processing each block.
#[derive(Params)]
pub struct FxOrderParams {
    #[id = "chr"] pub chorus:      IntParam,
    #[id = "dly"] pub delay:       IntParam,
    #[id = "shm"] pub shimmer:     IntParam,
    #[id = "gap"] pub gapper:      IntParam,
    #[id = "rev"] pub reverb:      IntParam,
    #[id = "dst"] pub distortion:  IntParam,
}

impl FxOrderParams {
    fn new() -> Self {
        let pos = |name, default| IntParam::new(name, default, IntRange::Linear { min: 1, max: 6 });
        Self {
            chorus:     pos("Chorus pos",     1),
            delay:      pos("Delay pos",      2),
            shimmer:    pos("Shimmer pos",    3),
            gapper:     pos("Gapper pos",     4),
            reverb:     pos("Reverb pos",     5),
            distortion: pos("Distortion pos", 6),
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

    #[nested(id_prefix = "shim", group = "Shimmer")]
    pub shimmer: Arc<ShimmerFxParams>,

    #[nested(id_prefix = "gap", group = "Gapper")]
    pub gapper: Arc<GapperFxParams>,

    #[nested(id_prefix = "arp", group = "Arpeggiator")]
    pub arp: Arc<ArpParams>,

    #[nested(id_prefix = "rev", group = "Reverb")]
    pub reverb: Arc<ReverbFxParams>,

    #[nested(id_prefix = "dist", group = "Distortion")]
    pub distortion: Arc<DistortionFxParams>,

    #[nested(id_prefix = "fxord", group = "FX Order")]
    pub fx_order: Arc<FxOrderParams>,

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
            shimmer: Arc::new(ShimmerFxParams::new()),
            gapper: Arc::new(GapperFxParams::new()),
            arp: Arc::new(ArpParams::new()),
            reverb: Arc::new(ReverbFxParams::new()),
            distortion: Arc::new(DistortionFxParams::new()),
            fx_order: Arc::new(FxOrderParams::new()),
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
