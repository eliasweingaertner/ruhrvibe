//! Stereo ping-pong delay with tone control.
//!
//! Each channel has its own delay line; feedback crosses sides so a hit
//! on L echoes on R, then L again, etc. A one-pole lowpass sits in the
//! feedback path so repeats get progressively darker — classic dub/tape
//! character.

/// Max delay in samples. 2 s at 192 kHz.
const BUFFER_SIZE: usize = 1 << 19; // 524288, ~2.7 s at 192 kHz
const BUFFER_MASK: usize = BUFFER_SIZE - 1;

pub struct Delay {
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
    write_pos: usize,
    lp_state_l: f32,
    lp_state_r: f32,
    sample_rate: f32,
}

impl Delay {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            buf_l: vec![0.0; BUFFER_SIZE],
            buf_r: vec![0.0; BUFFER_SIZE],
            write_pos: 0,
            lp_state_l: 0.0,
            lp_state_r: 0.0,
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
        self.lp_state_l = 0.0;
        self.lp_state_r = 0.0;
    }

    /// Process one stereo sample.
    /// `time_ms`, `feedback` (0–0.95), `tone` (0.05–1, 1=bright), `mix` (0–1).
    #[inline]
    pub fn process(
        &mut self,
        in_l: f32,
        in_r: f32,
        time_ms: f32,
        feedback: f32,
        tone: f32,
        mix: f32,
    ) -> (f32, f32) {
        let delay_samples = (time_ms * self.sample_rate * 0.001)
            .clamp(1.0, (BUFFER_SIZE - 1) as f32) as usize;
        let read_pos = (self.write_pos + BUFFER_SIZE - delay_samples) & BUFFER_MASK;

        let del_l = self.buf_l[read_pos];
        let del_r = self.buf_r[read_pos];

        // One-pole lowpass in the feedback path.
        let a = tone.clamp(0.01, 1.0);
        self.lp_state_l = self.lp_state_l * (1.0 - a) + del_l * a;
        self.lp_state_r = self.lp_state_r * (1.0 - a) + del_r * a;

        // Ping-pong: each side feeds back from the *other* side's tap.
        self.buf_l[self.write_pos] = in_l + self.lp_state_r * feedback;
        self.buf_r[self.write_pos] = in_r + self.lp_state_l * feedback;

        self.write_pos = (self.write_pos + 1) & BUFFER_MASK;

        let out_l = in_l + del_l * mix;
        let out_r = in_r + del_r * mix;
        (out_l, out_r)
    }
}
