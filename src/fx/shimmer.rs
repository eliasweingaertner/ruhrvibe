//! Shimmer delay — a ping-pong delay whose feedback path is pitch-shifted
//! up one octave on every pass.
//!
//! Echoes rise octave-by-octave until the fixed lowpass in the feedback
//! loop kills the top end, producing the characteristic ethereal
//! ascending cloud. Internally each channel owns a small grain-based
//! pitch shifter (two Hann-windowed grains with 50 % overlap, classic
//! delay-line varispeed).

/// Main delay-line buffer, per channel. ~2.7 s at 192 kHz.
const DELAY_BUFFER_SIZE: usize = 1 << 19;
const DELAY_BUFFER_MASK: usize = DELAY_BUFFER_SIZE - 1;

/// Grain length inside the pitch shifter. 1024 samples ≈ 23 ms at 48 kHz.
/// Shorter than this gets gritty; longer makes the shimmer smear.
const GRAIN_SIZE: f32 = 1024.0;

/// Pitch-shifter buffer. Must exceed `GRAIN_SIZE * (1 + max_pitch_ratio)`;
/// 4096 comfortably fits 1024 × 3 with a power-of-two mask.
const PITCH_BUFFER_SIZE: usize = 4096;
const PITCH_BUFFER_MASK: usize = PITCH_BUFFER_SIZE - 1;

/// Fixed one-pole LP coefficient in the feedback path. Lower = darker;
/// keeping it dark is what makes the shimmer feel musical rather than
/// turning into a dog-whistle over many feedback passes.
const FEEDBACK_LP_ALPHA: f32 = 0.35;

/// Two-grain overlap-add delay-line pitch shifter.
struct PitchShifter {
    buf: Vec<f32>,
    write_pos: usize,
    /// Phase 0..GRAIN_SIZE for each grain; advance by 1 per sample.
    phase_a: f32,
    phase_b: f32,
    /// Captured write position at the start of each grain.
    start_a: f32,
    start_b: f32,
}

impl PitchShifter {
    fn new() -> Self {
        let initial_start =
            (PITCH_BUFFER_SIZE as f32 - GRAIN_SIZE * 2.0).rem_euclid(PITCH_BUFFER_SIZE as f32);
        Self {
            buf: vec![0.0; PITCH_BUFFER_SIZE],
            write_pos: 0,
            phase_a: 0.0,
            phase_b: GRAIN_SIZE * 0.5,
            start_a: initial_start,
            start_b: initial_start,
        }
    }

    fn reset(&mut self) {
        self.buf.fill(0.0);
        self.write_pos = 0;
        self.phase_a = 0.0;
        self.phase_b = GRAIN_SIZE * 0.5;
        let initial_start =
            (PITCH_BUFFER_SIZE as f32 - GRAIN_SIZE * 2.0).rem_euclid(PITCH_BUFFER_SIZE as f32);
        self.start_a = initial_start;
        self.start_b = initial_start;
    }

    /// Pitch-shift one sample by `pitch_ratio` (2.0 = octave up).
    #[inline]
    fn process(&mut self, input: f32, pitch_ratio: f32) -> f32 {
        self.buf[self.write_pos] = input;

        self.phase_a += 1.0;
        self.phase_b += 1.0;

        if self.phase_a >= GRAIN_SIZE {
            self.phase_a = 0.0;
            self.start_a = (self.write_pos as f32 - GRAIN_SIZE * 2.0)
                .rem_euclid(PITCH_BUFFER_SIZE as f32);
        }
        if self.phase_b >= GRAIN_SIZE {
            self.phase_b = 0.0;
            self.start_b = (self.write_pos as f32 - GRAIN_SIZE * 2.0)
                .rem_euclid(PITCH_BUFFER_SIZE as f32);
        }

        let read_a =
            (self.start_a + self.phase_a * pitch_ratio).rem_euclid(PITCH_BUFFER_SIZE as f32);
        let read_b =
            (self.start_b + self.phase_b * pitch_ratio).rem_euclid(PITCH_BUFFER_SIZE as f32);

        let s_a = interp(&self.buf, read_a, PITCH_BUFFER_MASK);
        let s_b = interp(&self.buf, read_b, PITCH_BUFFER_MASK);

        let w_a = hann(self.phase_a / GRAIN_SIZE);
        let w_b = hann(self.phase_b / GRAIN_SIZE);

        self.write_pos = (self.write_pos + 1) & PITCH_BUFFER_MASK;

        s_a * w_a + s_b * w_b
    }
}

pub struct Shimmer {
    delay_l: Vec<f32>,
    delay_r: Vec<f32>,
    write_pos: usize,
    pitch_l: PitchShifter,
    pitch_r: PitchShifter,
    lp_l: f32,
    lp_r: f32,
    sample_rate: f32,
}

impl Shimmer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            delay_l: vec![0.0; DELAY_BUFFER_SIZE],
            delay_r: vec![0.0; DELAY_BUFFER_SIZE],
            write_pos: 0,
            pitch_l: PitchShifter::new(),
            pitch_r: PitchShifter::new(),
            lp_l: 0.0,
            lp_r: 0.0,
            sample_rate,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.reset();
    }

    pub fn reset(&mut self) {
        self.delay_l.fill(0.0);
        self.delay_r.fill(0.0);
        self.write_pos = 0;
        self.pitch_l.reset();
        self.pitch_r.reset();
        self.lp_l = 0.0;
        self.lp_r = 0.0;
    }

    /// Process one stereo sample.
    /// `time_ms` 1–2000, `feedback` 0–0.9, `mix` 0–1.
    #[inline]
    pub fn process(
        &mut self,
        in_l: f32,
        in_r: f32,
        time_ms: f32,
        feedback: f32,
        mix: f32,
    ) -> (f32, f32) {
        let delay_samples = (time_ms * self.sample_rate * 0.001)
            .clamp(1.0, (DELAY_BUFFER_SIZE - 1) as f32) as usize;
        let read_pos = (self.write_pos + DELAY_BUFFER_SIZE - delay_samples) & DELAY_BUFFER_MASK;

        let tap_l = self.delay_l[read_pos];
        let tap_r = self.delay_r[read_pos];

        // Pitch-shift the delayed tap up an octave (2× playback rate).
        let shifted_l = self.pitch_l.process(tap_l, 2.0);
        let shifted_r = self.pitch_r.process(tap_r, 2.0);

        // Dark one-pole LP in the feedback path; without this, each pass
        // adds an octave of content and the repeats become brittle.
        self.lp_l = self.lp_l * (1.0 - FEEDBACK_LP_ALPHA) + shifted_l * FEEDBACK_LP_ALPHA;
        self.lp_r = self.lp_r * (1.0 - FEEDBACK_LP_ALPHA) + shifted_r * FEEDBACK_LP_ALPHA;

        // Cross-feed feedback (ping-pong); each side echoes from the
        // other's shifted tap.
        self.delay_l[self.write_pos] = in_l + self.lp_r * feedback;
        self.delay_r[self.write_pos] = in_r + self.lp_l * feedback;

        self.write_pos = (self.write_pos + 1) & DELAY_BUFFER_MASK;

        // Output blends dry + shimmering (pitched, LP-ed) tap.
        let out_l = in_l + self.lp_l * mix;
        let out_r = in_r + self.lp_r * mix;
        (out_l, out_r)
    }
}

#[inline]
fn hann(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    0.5 - 0.5 * (x * std::f32::consts::TAU).cos()
}

#[inline]
fn interp(buf: &[f32], pos: f32, mask: usize) -> f32 {
    let i0 = (pos as usize) & mask;
    let i1 = (i0 + 1) & mask;
    let frac = pos - pos.floor();
    buf[i0] * (1.0 - frac) + buf[i1] * frac
}
