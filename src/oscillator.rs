//! Anti-aliased oscillator.
//!
//! Generates sine/saw/square/triangle waveforms. Saw and square are
//! band-limited using PolyBLEP (polynomial band-limited step) correction
//! to reduce aliasing at high frequencies. Triangle is generated via a
//! leaky integrator applied to a PolyBLEP square wave.

use crate::fast_math::exp2_fast;
use crate::params::Waveform;
use std::f32::consts::TAU;

pub struct Oscillator {
    /// Current phase in [0, 1).
    phase: f32,
    /// Phase increment per sample (frequency / sample_rate).
    phase_increment: f32,
    inv_sample_rate: f32,
    /// State for the triangle leaky integrator.
    triangle_state: f32,
    /// Simple noise PRNG state.
    noise_state: u32,
}

impl Oscillator {
    pub fn new_with_seed(sample_rate: f32, seed: u32) -> Self {
        Self {
            phase: 0.0,
            phase_increment: 0.0,
            inv_sample_rate: 1.0 / sample_rate,
            triangle_state: 0.0,
            noise_state: seed.max(1),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.inv_sample_rate = 1.0 / sample_rate;
    }

    #[inline]
    pub fn set_frequency(&mut self, freq_hz: f32) {
        self.phase_increment = freq_hz * self.inv_sample_rate;
        // Clamp to avoid instability near Nyquist.
        if self.phase_increment > 0.49 {
            self.phase_increment = 0.49;
        }
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.triangle_state = 0.0;
    }

    /// Generate next sample, advancing phase.
    #[inline]
    pub fn next_sample(&mut self, waveform: Waveform) -> f32 {
        if waveform == Waveform::Noise {
            return self.generate_noise();
        }

        let sample = match waveform {
            Waveform::Sine => self.generate_sine(),
            Waveform::Saw => self.generate_saw(),
            Waveform::Square => self.generate_square(),
            Waveform::Triangle => self.generate_triangle(),
            Waveform::Noise => unreachable!(),
        };

        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }

    #[inline]
    fn generate_sine(&self) -> f32 {
        (self.phase * TAU).sin()
    }

    #[inline]
    fn poly_blep(&self, t: f32) -> f32 {
        let dt = self.phase_increment;
        if t < dt {
            let t = t / dt;
            2.0 * t - t * t - 1.0
        } else if t > 1.0 - dt {
            let t = (t - 1.0) / dt;
            t * t + 2.0 * t + 1.0
        } else {
            0.0
        }
    }

    #[inline]
    fn generate_saw(&self) -> f32 {
        let naive = 2.0 * self.phase - 1.0;
        naive - self.poly_blep(self.phase)
    }

    #[inline]
    fn generate_square(&self) -> f32 {
        let naive = if self.phase < 0.5 { 1.0 } else { -1.0 };
        let blep_up = self.poly_blep(self.phase);
        let shifted = (self.phase + 0.5) % 1.0;
        let blep_down = self.poly_blep(shifted);
        naive + blep_up - blep_down
    }

    #[inline]
    fn generate_noise(&mut self) -> f32 {
        let mut x = self.noise_state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.noise_state = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    #[inline]
    fn generate_triangle(&mut self) -> f32 {
        let square = self.generate_square();
        self.triangle_state =
            self.phase_increment * 4.0 * square + (1.0 - self.phase_increment * 0.5) * self.triangle_state;
        self.triangle_state.clamp(-1.0, 1.0)
    }
}

/// Convert a MIDI note number (fractional, for pitch env) to frequency in Hz.
/// Uses fast exp2 approximation. A4 (note 69) = 440 Hz.
#[inline]
pub fn midi_note_to_freq(note: f32) -> f32 {
    440.0 * exp2_fast((note - 69.0) * (1.0 / 12.0))
}
