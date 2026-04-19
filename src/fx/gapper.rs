//! Host-synced rhythmic gate ("trance gate").
//!
//! Chops the signal on/off in sync with the host tempo. When the host
//! provides a beat position we lock to it directly (same pattern position
//! every time the DAW loops around); when it doesn't, we free-run using
//! the host tempo alone.

pub struct Gapper {
    /// Free-running phase in [0, 1). Used when the host doesn't provide a
    /// beat position; also kept in sync when the host does, so switching
    /// between the two is seamless.
    phase: f32,
    sample_rate: f32,
}

impl Gapper {
    pub fn new(sample_rate: f32) -> Self {
        Self { phase: 0.0, sample_rate }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.phase = 0.0;
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    /// Process one stereo sample.
    ///
    /// * `host_beats` — current absolute playhead position in beats from the
    ///   host. `Some` → the gate locks to it; `None` → free-run using `bpm`.
    /// * `beats_per_cycle` — musical length of one gate cycle (1/16 = 0.25).
    /// * `bpm` — host tempo, used in free-run mode.
    /// * `duty` — 0..1 fraction of each cycle that's "open".
    /// * `smooth` — 0..0.5 edge softness (fade window at each transition).
    /// * `depth` — 0..1 attenuation during the closed portion (1 = full mute).
    #[inline]
    pub fn process(
        &mut self,
        in_l: f32,
        in_r: f32,
        host_beats: Option<f64>,
        beats_per_cycle: f32,
        bpm: f32,
        duty: f32,
        smooth: f32,
        depth: f32,
    ) -> (f32, f32) {
        let phase = match host_beats {
            Some(b) => {
                let p = (b / beats_per_cycle as f64).rem_euclid(1.0) as f32;
                self.phase = p;
                p
            }
            None => {
                let cycles_per_sample =
                    bpm / (60.0 * beats_per_cycle.max(1e-4) * self.sample_rate);
                self.phase += cycles_per_sample;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                self.phase
            }
        };

        let gate = gate_value(phase, duty, smooth);
        let gain = 1.0 - depth * (1.0 - gate);
        (in_l * gain, in_r * gain)
    }
}

#[inline]
fn gate_value(phase: f32, duty: f32, smooth: f32) -> f32 {
    let duty = duty.clamp(0.0, 1.0);
    if duty <= 0.0 {
        return 0.0;
    }
    if duty >= 1.0 {
        return 1.0;
    }

    let max_edge = duty.min(1.0 - duty);
    let edge = (smooth * 0.5).min(max_edge);

    if edge < 1e-5 {
        return if phase < duty { 1.0 } else { 0.0 };
    }

    if phase < edge {
        smoothstep(phase / edge)
    } else if phase < duty - edge {
        1.0
    } else if phase < duty + edge {
        smoothstep((duty + edge - phase) / (2.0 * edge))
    } else {
        0.0
    }
}

#[inline]
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
