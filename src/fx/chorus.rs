//! Classic 2-voice stereo chorus.
//!
//! A single modulated delay line per channel. The left and right LFOs
//! are 90° apart so the modulation reads different delay positions on
//! each side, producing stereo width even from a mono input.

use std::f32::consts::TAU;

/// Max delay buffer size in samples. Sized for 192 kHz + full modulation
/// swing (base 7 ms + depth 5 ms = 12 ms ≈ 2304 samples at 192 kHz). We
/// round up to the next power-of-two for cheap modulo via mask.
const BUFFER_SIZE: usize = 4096;
const BUFFER_MASK: usize = BUFFER_SIZE - 1;

pub struct Chorus {
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
    write_pos: usize,
    lfo_phase: f32,
    sample_rate: f32,
}

impl Chorus {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            buf_l: vec![0.0; BUFFER_SIZE],
            buf_r: vec![0.0; BUFFER_SIZE],
            write_pos: 0,
            lfo_phase: 0.0,
            sample_rate,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.reset();
    }

    pub fn reset(&mut self) {
        self.buf_l.fill(0.0);
        self.buf_r.fill(0.0);
        self.write_pos = 0;
        self.lfo_phase = 0.0;
    }

    /// Process one stereo sample.
    /// `rate_hz`, `depth` (0–1), `mix` (0–1) are all per-sample scalars.
    #[inline]
    pub fn process(&mut self, in_l: f32, in_r: f32, rate_hz: f32, depth: f32, mix: f32) -> (f32, f32) {
        let lfo_l = (self.lfo_phase * TAU).sin();
        let lfo_r = ((self.lfo_phase + 0.25) * TAU).sin();

        self.lfo_phase += rate_hz / self.sample_rate;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }

        // Base 7 ms delay + depth-scaled ±5 ms modulation.
        let ms_to_samples = self.sample_rate * 0.001;
        let base = 7.0 * ms_to_samples;
        let swing = depth * 5.0 * ms_to_samples;
        let delay_l = base + lfo_l * swing;
        let delay_r = base + lfo_r * swing;

        self.buf_l[self.write_pos] = in_l;
        self.buf_r[self.write_pos] = in_r;

        let del_l = read_interp(&self.buf_l, self.write_pos, delay_l);
        let del_r = read_interp(&self.buf_r, self.write_pos, delay_r);

        self.write_pos = (self.write_pos + 1) & BUFFER_MASK;

        let out_l = in_l * (1.0 - mix) + del_l * mix;
        let out_r = in_r * (1.0 - mix) + del_r * mix;
        (out_l, out_r)
    }
}

/// Linear-interpolated read from a power-of-two delay buffer.
#[inline]
fn read_interp(buf: &[f32], write_pos: usize, delay_samples: f32) -> f32 {
    let read_pos_f = (write_pos as f32 + BUFFER_SIZE as f32 - delay_samples).max(0.0);
    let i0 = (read_pos_f as usize) & BUFFER_MASK;
    let i1 = (i0 + 1) & BUFFER_MASK;
    let frac = read_pos_f - read_pos_f.floor();
    buf[i0] * (1.0 - frac) + buf[i1] * frac
}
