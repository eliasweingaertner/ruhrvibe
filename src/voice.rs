//! A single polyphonic voice.
//!
//! Owns 2 oscillator banks (each up to MAX_UNISON copies for unison),
//! 2 filter slots (in series), a pitch envelope, and 3 ADSR envelopes
//! (amp + 2 filter envelopes). DSP state is per-voice; parameters are
//! passed in each process call as scalar values already read from
//! smoothers at the current sample position.

use crate::envelope::Envelope;
use crate::fast_math::{exp2_fast, INV_SQRT};
use crate::filter::{SvfCoeffs, SvfFilter};
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
    pub pan: f32,
    pub stereo_spread: f32,
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

/// Precomputed per-sample osc bank values that don't depend on the voice's
/// note. Shared across all voices playing this bank so we only pay the
/// `exp2_fast` / pan math once per sample, not once per voice per unison.
#[derive(Clone, Copy)]
pub struct OscBankPrecomp {
    pub n: usize,
    /// 2^(octave + detune_cents/1200). Scales `base_freq` to the bank center.
    pub octave_detune_ratio: f32,
    /// 2^(spread_factor * t_i) per unison voice. Index 0..n is valid.
    pub spread_ratios: [f32; MAX_UNISON],
    /// Per-unison-voice stereo pan gains (left/right). Index 0..n is valid.
    pub voice_pans_l: [f32; MAX_UNISON],
    pub voice_pans_r: [f32; MAX_UNISON],
    /// Bank-wide pan applied on top of unison spread.
    pub bank_pan_l: f32,
    pub bank_pan_r: f32,
    /// level * 1/sqrt(n) — amplitude per unison voice.
    pub norm: f32,
}

impl OscBankPrecomp {
    pub fn compute(p: &OscVoiceParams) -> Self {
        let n = (p.unison_voices as usize).clamp(1, MAX_UNISON);
        let octave_detune_ratio =
            exp2_fast(p.octave as f32 + p.detune_cents * (1.0 / 1200.0));
        let (bank_pan_l, bank_pan_r) = center_unity_pan(p.pan);

        let mut spread_ratios = [1.0f32; MAX_UNISON];
        let mut voice_pans_l = [1.0f32; MAX_UNISON];
        let mut voice_pans_r = [1.0f32; MAX_UNISON];

        let norm = if n == 1 {
            p.level
        } else {
            let spread_factor = p.unison_spread * (1.0 / 1200.0);
            let inv_n_minus_1 = 1.0 / (n - 1) as f32;
            for i in 0..n {
                let t = (i as f32 * inv_n_minus_1) * 2.0 - 1.0;
                spread_ratios[i] = exp2_fast(spread_factor * t);
                let voice_pan = (t * p.stereo_spread).clamp(-1.0, 1.0);
                let (vl, vr) = center_unity_pan(voice_pan);
                voice_pans_l[i] = vl;
                voice_pans_r[i] = vr;
            }
            p.level * INV_SQRT[n]
        };

        Self {
            n,
            octave_detune_ratio,
            spread_ratios,
            voice_pans_l,
            voice_pans_r,
            bank_pan_l,
            bank_pan_r,
            norm,
        }
    }
}

/// Bundled parameters passed to a voice for one sample of processing.
#[derive(Clone, Copy)]
pub struct VoiceParams {
    pub osc1: OscVoiceParams,
    pub osc2: OscVoiceParams,
    pub osc1_pre: OscBankPrecomp,
    pub osc2_pre: OscBankPrecomp,
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
    // Stereo filter pair per slot — each channel has its own state so
    // stereo content survives the filter.
    filter1_l: SvfFilter,
    filter1_r: SvfFilter,
    filter2_l: SvfFilter,
    filter2_r: SvfFilter,
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
            filter1_l: SvfFilter::new(sample_rate),
            filter1_r: SvfFilter::new(sample_rate),
            filter2_l: SvfFilter::new(sample_rate),
            filter2_r: SvfFilter::new(sample_rate),
            amp_env: Envelope::new(sample_rate),
            filter1_env: Envelope::new(sample_rate),
            filter2_env: Envelope::new(sample_rate),
            pitch_env: Envelope::new(sample_rate),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        for o in &mut self.osc1 { o.set_sample_rate(sample_rate); }
        for o in &mut self.osc2 { o.set_sample_rate(sample_rate); }
        self.filter1_l.set_sample_rate(sample_rate);
        self.filter1_r.set_sample_rate(sample_rate);
        self.filter2_l.set_sample_rate(sample_rate);
        self.filter2_r.set_sample_rate(sample_rate);
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
        self.filter1_l.reset();
        self.filter1_r.reset();
        self.filter2_l.reset();
        self.filter2_r.reset();
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
        self.filter1_l.reset();
        self.filter1_r.reset();
        self.filter2_l.reset();
        self.filter2_r.reset();
        self.amp_env.reset();
        self.filter1_env.reset();
        self.filter2_env.reset();
        self.pitch_env.reset();
    }

    /// Process a single sample and return the voice output (L, R).
    #[inline]
    pub fn process(&mut self, params: &VoiceParams) -> (f32, f32) {
        if !self.is_active() {
            return (0.0, 0.0);
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

        // Oscillator 1 + 2, each produces (L, R).
        let (mut mix_l, mut mix_r) = (0.0f32, 0.0f32);
        if params.osc1.enabled {
            let (l, r) = Self::process_osc_bank(
                &mut self.osc1,
                base_freq,
                params.osc1.waveform,
                &params.osc1_pre,
            );
            mix_l += l;
            mix_r += r;
        }
        if params.osc2.enabled {
            let (l, r) = Self::process_osc_bank(
                &mut self.osc2,
                base_freq,
                params.osc2.waveform,
                &params.osc2_pre,
            );
            mix_l += l;
            mix_r += r;
        }

        // Filter 1 (per-channel). SVF is linear, but we keep separate state
        // so stereo content survives the filter (phase/resonance behavior).
        // Coefficients are computed once and shared across L/R to avoid
        // redundant tan() calls.
        let (mut sig_l, mut sig_r) = (mix_l, mix_r);
        if params.filter1.enabled {
            let modulated_cutoff = params.filter1.cutoff
                * exp2_fast(f1_env * params.filter1.env_amount * 4.0);
            let coeffs = SvfCoeffs::compute(
                modulated_cutoff,
                params.filter1.resonance,
                params.filter1.drive,
                params.filter1.filter_type,
                self.filter1_l.inv_sample_rate(),
                self.filter1_l.half_sample_rate(),
            );
            sig_l = self.filter1_l.process_coeffs(sig_l, &coeffs);
            sig_r = self.filter1_r.process_coeffs(sig_r, &coeffs);
        }

        // Filter 2 (per-channel).
        if params.filter2.enabled {
            let modulated_cutoff = params.filter2.cutoff
                * exp2_fast(f2_env * params.filter2.env_amount * 4.0);
            let coeffs = SvfCoeffs::compute(
                modulated_cutoff,
                params.filter2.resonance,
                params.filter2.drive,
                params.filter2.filter_type,
                self.filter2_l.inv_sample_rate(),
                self.filter2_l.half_sample_rate(),
            );
            sig_l = self.filter2_l.process_coeffs(sig_l, &coeffs);
            sig_r = self.filter2_r.process_coeffs(sig_r, &coeffs);
        }

        let gain = amp * self.velocity;
        (sig_l * gain, sig_r * gain)
    }

    /// Process an oscillator bank with unison detuning. Returns (L, R) with
    /// unison voices spread across the stereo field and the bank's pan applied.
    /// All frequency ratios and pan gains come from the precomputed bank state
    /// so this path avoids per-voice `exp2_fast` and trig calls.
    #[inline]
    fn process_osc_bank(
        oscs: &mut [Oscillator; MAX_UNISON],
        base_freq: f32,
        waveform: Waveform,
        pre: &OscBankPrecomp,
    ) -> (f32, f32) {
        let center_freq = base_freq * pre.octave_detune_ratio;

        if pre.n == 1 {
            oscs[0].set_frequency(center_freq);
            let s = oscs[0].next_sample(waveform) * pre.norm;
            return (s * pre.bank_pan_l, s * pre.bank_pan_r);
        }

        let mut sum_l = 0.0f32;
        let mut sum_r = 0.0f32;
        for i in 0..pre.n {
            let freq = center_freq * pre.spread_ratios[i];
            oscs[i].set_frequency(freq);
            let s = oscs[i].next_sample(waveform) * pre.norm;
            sum_l += s * pre.voice_pans_l[i];
            sum_r += s * pre.voice_pans_r[i];
        }
        (sum_l * pre.bank_pan_l, sum_r * pre.bank_pan_r)
    }
}

/// Constant-power pan law, normalized so pan=0 returns (1.0, 1.0).
/// pan=-1 → (sqrt(2), 0), pan=1 → (0, sqrt(2)).
#[inline]
fn center_unity_pan(pan: f32) -> (f32, f32) {
    let theta = (pan.clamp(-1.0, 1.0) + 1.0) * std::f32::consts::FRAC_PI_4;
    (theta.cos() * std::f32::consts::SQRT_2,
     theta.sin() * std::f32::consts::SQRT_2)
}
