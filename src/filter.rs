//! State Variable Filter (SVF) in the Cytomic/Simper style.
//!
//! Zero-delay-feedback (ZDF) topology, stable at high resonance, produces
//! LP, HP, BP, and notch outputs from a single computation. Each voice
//! owns its own filter instance with independent state.

use crate::params::FilterType;
use std::f32::consts::PI;

pub struct SvfFilter {
    ic1eq: f32,
    ic2eq: f32,
    inv_sample_rate: f32,
    half_sample_rate: f32,
}

impl SvfFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            ic1eq: 0.0,
            ic2eq: 0.0,
            inv_sample_rate: 1.0 / sample_rate,
            half_sample_rate: sample_rate * 0.49,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.inv_sample_rate = 1.0 / sample_rate;
        self.half_sample_rate = sample_rate * 0.49;
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    #[inline]
    pub fn process(
        &mut self,
        input: f32,
        cutoff_hz: f32,
        resonance: f32,
        drive: f32,
        filter_type: FilterType,
    ) -> f32 {
        let driven = if drive > 1.0 {
            fast_tanh(input * drive) / fast_sqrt(drive)
        } else {
            input
        };

        // Clamp cutoff.
        let cutoff = if cutoff_hz < 20.0 {
            20.0
        } else if cutoff_hz > self.half_sample_rate {
            self.half_sample_rate
        } else {
            cutoff_hz
        };

        // Pre-warp: g = tan(pi * cutoff / sr).
        let g = (PI * cutoff * self.inv_sample_rate).tan();

        let k = (2.0 - 2.0 * resonance.min(0.995)).max(0.01);

        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        let v3 = driven - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;

        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        match filter_type {
            FilterType::LowPass => v2,
            FilterType::HighPass => driven - k * v1 - v2,
            FilterType::BandPass => v1,
            FilterType::Notch => {
                let high = driven - k * v1 - v2;
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
