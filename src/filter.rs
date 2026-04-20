//! State Variable Filter (SVF) in the Cytomic/Simper style.
//!
//! Zero-delay-feedback (ZDF) topology, stable at high resonance, produces
//! LP, HP, BP, and notch outputs from a single computation. Each voice
//! owns its own filter instance with independent state.

use crate::params::FilterType;

/// Pre-computed filter coefficients, shared across both L/R channels of a
/// single filter slot for one sample. Holding the expensive tan() and
/// 1/(1+g*(g+k)) outside the filter struct means they're computed once per
/// voice-per-slot instead of once per voice-per-slot-per-channel — and the
/// same instance can be shared across every voice in the pool when the
/// filter envelope isn't modulating cutoff.
#[derive(Clone, Copy)]
pub struct SvfCoeffs {
    pub k: f32,
    pub a1: f32,
    pub a2: f32,
    pub a3: f32,
    pub drive: f32,
    pub inv_sqrt_drive: f32,
    pub filter_type: FilterType,
}

impl SvfCoeffs {
    #[inline]
    pub fn compute(
        cutoff_hz: f32,
        resonance: f32,
        drive: f32,
        filter_type: FilterType,
        pi_over_fs: f32,
        nyquist: f32,
    ) -> Self {
        let cutoff = cutoff_hz.clamp(20.0, nyquist);
        let g = (pi_over_fs * cutoff).tan();
        let k = (2.0 - 2.0 * resonance.min(0.995)).max(0.01);
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let inv_sqrt_drive = if drive > 1.0 { fast_sqrt(drive).recip() } else { 1.0 };
        Self { k, a1, a2, a3, drive, inv_sqrt_drive, filter_type }
    }
}

pub struct SvfFilter {
    ic1eq: f32,
    ic2eq: f32,
}

impl SvfFilter {
    pub fn new() -> Self {
        Self { ic1eq: 0.0, ic2eq: 0.0 }
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    /// Process one sample using precomputed coefficients.
    #[inline]
    pub fn process_coeffs(&mut self, input: f32, c: &SvfCoeffs) -> f32 {
        let driven = if c.drive > 1.0 {
            fast_tanh(input * c.drive) * c.inv_sqrt_drive
        } else {
            input
        };

        let v3 = driven - self.ic2eq;
        let v1 = c.a1 * self.ic1eq + c.a2 * v3;
        let v2 = self.ic2eq + c.a2 * self.ic1eq + c.a3 * v3;

        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        match c.filter_type {
            FilterType::LowPass => v2,
            FilterType::HighPass => driven - c.k * v1 - v2,
            FilterType::BandPass => v1,
            FilterType::Notch => {
                let high = driven - c.k * v1 - v2;
                v2 + high
            }
        }
    }
}

/// Cheap approximation of tanh for soft saturation.
#[inline]
fn fast_tanh(x: f32) -> f32 {
    let x2 = x * x;
    x * (27.0 + x2) / (27.0 + 9.0 * x2)
}

/// Fast inverse square root approximation (for drive normalization).
#[inline]
fn fast_sqrt(x: f32) -> f32 {
    // Use the bit trick for a rough sqrt, then one Newton iteration.
    let i = f32::to_bits(x);
    let i = 0x1FBD1DF5 + (i >> 1);
    let y = f32::from_bits(i);
    // One Newton-Raphson step.
    0.5 * (y + x / y)
}
