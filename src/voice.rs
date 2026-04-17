//! A single polyphonic voice.
//!
//! Owns 2 oscillator banks (each up to MAX_UNISON copies for unison),
//! 2 filter slots (in series), a pitch envelope, and 3 ADSR envelopes
//! (amp + 2 filter envelopes). DSP state is per-voice; parameters are
//! passed in each process call as scalar values already read from
//! smoothers at the current sample position.

use crate::envelope::Envelope;
use crate::fast_math::{exp2_fast, INV_SQRT};
use crate::filter::SvfFilter;
use crate::oscillator::{midi_note_to_freq, Oscillator};
use crate::params::{FilterType, Waveform};

/// Maximum unison copies per oscillator.
pub const MAX_UNISON: usize = 7;

/// Pre-computed per-sample parameter values for a single oscillator.
#[derive(Clone, Copy)]
pub struct OscVoiceParams {
    pub waveform: Waveform,
    pub level: f32,
    pub detune_cents: f32,
    pub octave: i32,
    pub enabled: bool,
    pub unison_voices: i32,
    pub unison_spread: f32,
}

/// Pre-computed per-sample parameter values for a single filter slot.
#[derive(Clone, Copy)]
pub struct FilterVoiceParams {
    pub filter_type: FilterType,
    pub cutoff: f32,
    pub resonance: f32,
    pub drive: f32,
    pub env_amount: f32,
    pub enabled: bool,
}

/// Pre-computed per-sample ADSR values.
#[derive(Clone, Copy)]
pub struct EnvelopeVoiceParams {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

/// Pitch envelope parameters.
#[derive(Clone, Copy)]
pub struct PitchEnvVoiceParams {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub amount: f32,
}

/// Bundled parameters passed to a voice for one sample of processing.
#[derive(Clone, Copy)]
pub struct VoiceParams {
    pub osc1: OscVoiceParams,
    pub osc2: OscVoiceParams,
    pub filter1: FilterVoiceParams,
    pub filter2: FilterVoiceParams,
    pub amp_env: EnvelopeVoiceParams,
    pub filter1_env: EnvelopeVoiceParams,
    pub filter2_env: EnvelopeVoiceParams,
    pub pitch_env: PitchEnvVoiceParams,
}

pub struct Voice {
    pub note: u8,
    pub velocity: f32,
    osc1: [Oscillator; MAX_UNISON],
    osc2: [Oscillator; MAX_UNISON],
    filter1: SvfFilter,
    filter2: SvfFilter,
    amp_env: Envelope,
    filter1_env: Envelope,
    filter2_env: Envelope,
    pitch_env: Envelope,
}

impl Voice {
    pub fn new(sample_rate: f32) -> Self {
        let osc1 = std::array::from_fn(|i| {
            Oscillator::new_with_seed(sample_rate, 12345 + (i as u32) * 7919)
        });
        let osc2 = std::array::from_fn(|i| {
            Oscillator::new_with_seed(sample_rate, 54321 + (i as u32) * 6271)
        });
        Self {
            note: 0,
            velocity: 0.0,
            osc1,
            osc2,
            filter1: SvfFilter::new(sample_rate),
            filter2: SvfFilter::new(sample_rate),
            amp_env: Envelope::new(sample_rate),
            filter1_env: Envelope::new(sample_rate),
            filter2_env: Envelope::new(sample_rate),
            pitch_env: Envelope::new(sample_rate),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        for o in &mut self.osc1 { o.set_sample_rate(sample_rate); }
        for o in &mut self.osc2 { o.set_sample_rate(sample_rate); }
        self.filter1.set_sample_rate(sample_rate);
        self.filter2.set_sample_rate(sample_rate);
        self.amp_env.set_sample_rate(sample_rate);
        self.filter1_env.set_sample_rate(sample_rate);
        self.filter2_env.set_sample_rate(sample_rate);
        self.pitch_env.set_sample_rate(sample_rate);
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        self.note = note;
        self.velocity = velocity;
        for o in &mut self.osc1 { o.reset(); }
        for o in &mut self.osc2 { o.reset(); }
        self.filter1.reset();
        self.filter2.reset();
        self.amp_env.trigger();
        self.filter1_env.trigger();
        self.filter2_env.trigger();
        self.pitch_env.trigger();
    }

    pub fn note_off(&mut self) {
        self.amp_env.release();
        self.filter1_env.release();
        self.filter2_env.release();
        self.pitch_env.release();
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        !self.amp_env.is_idle()
    }

    #[inline]
    pub fn amp_level(&self) -> f32 {
        self.amp_env.level()
    }

    pub fn reset(&mut self) {
        for o in &mut self.osc1 { o.reset(); }
        for o in &mut self.osc2 { o.reset(); }
        self.filter1.reset();
        self.filter2.reset();
        self.amp_env.reset();
        self.filter1_env.reset();
        self.filter2_env.reset();
        self.pitch_env.reset();
    }

    /// Process a single sample and return the voice output (mono).
    #[inline]
    pub fn process(&mut self, params: &VoiceParams) -> f32 {
        if !self.is_active() {
            return 0.0;
        }

        // Advance envelopes.
        let amp = self.amp_env.next_sample(
            params.amp_env.attack,
            params.amp_env.decay,
            params.amp_env.sustain,
            params.amp_env.release,
        );
        let f1_env = self.filter1_env.next_sample(
            params.filter1_env.attack,
            params.filter1_env.decay,
            params.filter1_env.sustain,
            params.filter1_env.release,
        );
        let f2_env = self.filter2_env.next_sample(
            params.filter2_env.attack,
            params.filter2_env.decay,
            params.filter2_env.sustain,
            params.filter2_env.release,
        );
        let pitch_env_val = self.pitch_env.next_sample(
            params.pitch_env.attack,
            params.pitch_env.decay,
            params.pitch_env.sustain,
            params.pitch_env.release,
        );

        // Pitch envelope modulates base note in semitones.
        let pitch_offset_semitones = pitch_env_val * params.pitch_env.amount;
        let base_freq = midi_note_to_freq(self.note as f32 + pitch_offset_semitones);

        // Oscillator 1 (unison)
        let mut mix = 0.0;
        if params.osc1.enabled {
            mix += Self::process_osc_bank(
                &mut self.osc1,
                base_freq,
                &params.osc1,
            );
        }

        // Oscillator 2 (unison)
        if params.osc2.enabled {
            mix += Self::process_osc_bank(
                &mut self.osc2,
                base_freq,
                &params.osc2,
            );
        }

        // Filter 1 (if enabled)
        let mut signal = mix;
        if params.filter1.enabled {
            let modulated_cutoff = params.filter1.cutoff
                * exp2_fast(f1_env * params.filter1.env_amount * 4.0);
            signal = self.filter1.process(
                signal,
                modulated_cutoff,
                params.filter1.resonance,
                params.filter1.drive,
                params.filter1.filter_type,
            );
        }

        // Filter 2 (if enabled)
        if params.filter2.enabled {
            let modulated_cutoff = params.filter2.cutoff
                * exp2_fast(f2_env * params.filter2.env_amount * 4.0);
            signal = self.filter2.process(
                signal,
                modulated_cutoff,
                params.filter2.resonance,
                params.filter2.drive,
                params.filter2.filter_type,
            );
        }

        // Amplitude envelope and velocity scaling.
        signal * amp * self.velocity
    }

    /// Process an oscillator bank with unison detuning.
    #[inline]
    fn process_osc_bank(
        oscs: &mut [Oscillator; MAX_UNISON],
        base_freq: f32,
        p: &OscVoiceParams,
    ) -> f32 {
        let n = (p.unison_voices as usize).clamp(1, MAX_UNISON);

        // Compute center frequency: base * 2^octave * 2^(detune_cents/1200).
        // Combine octave + detune into one exp2 call.
        let center_freq = base_freq
            * exp2_fast(p.octave as f32 + p.detune_cents * (1.0 / 1200.0));

        if n == 1 {
            oscs[0].set_frequency(center_freq);
            return oscs[0].next_sample(p.waveform) * p.level;
        }

        // Spread unison voices symmetrically around center_freq.
        // Pre-compute the spread factor: spread_cents / 1200.
        let spread_factor = p.unison_spread * (1.0 / 1200.0);
        let inv_n_minus_1 = 1.0 / (n - 1) as f32;
        let mut sum = 0.0f32;
        for i in 0..n {
            let t = (i as f32 * inv_n_minus_1) * 2.0 - 1.0;
            let freq = center_freq * exp2_fast(spread_factor * t);
            oscs[i].set_frequency(freq);
            sum += oscs[i].next_sample(p.waveform);
        }
        // Use precomputed 1/sqrt(n) lookup instead of runtime sqrt.
        sum * p.level * INV_SQRT[n]
    }
}
