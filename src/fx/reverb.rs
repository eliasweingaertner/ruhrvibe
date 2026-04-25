//! Freeverb-style plate reverb.
//!
//! 8 parallel feedback comb filters (with damping LPF) feed 4 series allpass
//! filters per channel. Buffer sizes are scaled from the canonical 44100 Hz
//! tuning so the reverb time is consistent at any sample rate.

struct CombFilter {
    buf: Vec<f32>,
    pos: usize,
    filterstore: f32,
}

impl CombFilter {
    fn new(size: usize) -> Self {
        Self { buf: vec![0.0; size.max(1)], pos: 0, filterstore: 0.0 }
    }

    #[inline]
    fn process(&mut self, input: f32, feedback: f32, damp1: f32, damp2: f32) -> f32 {
        let out = self.buf[self.pos];
        self.filterstore = out * damp2 + self.filterstore * damp1;
        self.buf[self.pos] = input + self.filterstore * feedback;
        self.pos += 1;
        if self.pos >= self.buf.len() { self.pos = 0; }
        out
    }

    fn reset(&mut self) { self.buf.fill(0.0); self.pos = 0; self.filterstore = 0.0; }
}

struct AllpassFilter {
    buf: Vec<f32>,
    pos: usize,
}

impl AllpassFilter {
    fn new(size: usize) -> Self {
        Self { buf: vec![0.0; size.max(1)], pos: 0 }
    }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let bufout = self.buf[self.pos];
        let output = -input + bufout;
        self.buf[self.pos] = input + bufout * 0.5;
        self.pos += 1;
        if self.pos >= self.buf.len() { self.pos = 0; }
        output
    }

    fn reset(&mut self) { self.buf.fill(0.0); self.pos = 0; }
}

// Canonical Freeverb buffer sizes (samples at 44100 Hz).
const COMB_TUNING:    [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASS_TUNING: [usize; 4] = [556, 441, 341, 225];
const STEREO_SPREAD:  usize = 23;

pub struct Reverb {
    combs_l:    Vec<CombFilter>,
    combs_r:    Vec<CombFilter>,
    allpasses_l: Vec<AllpassFilter>,
    allpasses_r: Vec<AllpassFilter>,
}

impl Reverb {
    pub fn new(sample_rate: f32) -> Self {
        let scale = |s: usize| ((s as f32 * sample_rate / 44100.0).round() as usize).max(1);
        Self {
            combs_l:    COMB_TUNING.iter().map(|&s| CombFilter::new(scale(s))).collect(),
            combs_r:    COMB_TUNING.iter().map(|&s| CombFilter::new(scale(s + STEREO_SPREAD))).collect(),
            allpasses_l: ALLPASS_TUNING.iter().map(|&s| AllpassFilter::new(scale(s))).collect(),
            allpasses_r: ALLPASS_TUNING.iter().map(|&s| AllpassFilter::new(scale(s + STEREO_SPREAD))).collect(),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        *self = Reverb::new(sample_rate);
    }

    pub fn reset(&mut self) {
        for c in &mut self.combs_l    { c.reset(); }
        for c in &mut self.combs_r    { c.reset(); }
        for a in &mut self.allpasses_l { a.reset(); }
        for a in &mut self.allpasses_r { a.reset(); }
    }

    #[inline]
    pub fn process(
        &mut self,
        in_l: f32,
        in_r: f32,
        room_size: f32,
        damping: f32,
        width: f32,
        mix: f32,
    ) -> (f32, f32) {
        // Wider range: 0→very short (0.05), 1→near-infinite (0.995).
        // Capped at 0.995 to avoid unbounded growth.
        let feedback = (0.05 + room_size * 0.945).min(0.995);
        // Wider damping range: 0→fully bright, 1→very dark.
        let damp1    = damping * 0.85;
        let damp2    = 1.0 - damp1;
        let input    = (in_l + in_r) * 0.015;

        let mut out_l = 0.0f32;
        let mut out_r = 0.0f32;
        for (cl, cr) in self.combs_l.iter_mut().zip(self.combs_r.iter_mut()) {
            out_l += cl.process(input, feedback, damp1, damp2);
            out_r += cr.process(input, feedback, damp1, damp2);
        }
        for (al, ar) in self.allpasses_l.iter_mut().zip(self.allpasses_r.iter_mut()) {
            out_l = al.process(out_l);
            out_r = ar.process(out_r);
        }

        // Stereo width matrix + wet/dry blend.
        let w1 = mix * 3.0 * (0.5 + width * 0.5);
        let w2 = mix * 3.0 * (0.5 - width * 0.5);
        let dry = 1.0 - mix;
        (in_l * dry + out_l * w1 + out_r * w2,
         in_r * dry + out_r * w1 + out_l * w2)
    }
}
