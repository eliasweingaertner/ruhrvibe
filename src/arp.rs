//! Host-synced step arpeggiator.
//!
//! Consumes held MIDI notes and emits `note_on` / `note_off` events in a
//! tempo-locked rhythm. Locks to the host beat position when provided;
//! free-runs from tempo alone when the host doesn't expose one. The synth
//! routes these events to the voice pool the same way it handles MIDI.

use crate::params::{ArpPattern, ArpRoot, ArpScale};

/// Snap `pitch` to the nearest pitch-class allowed by `scale` (rooted at `root`).
/// Ties go to the lower neighbor. Returns `pitch` unchanged when scale is `Off`.
fn snap_to_scale(pitch: u8, scale: ArpScale, root: ArpRoot) -> u8 {
    let mask = scale.mask();
    if mask == 0xFFF {
        return pitch;
    }
    let root_pc = root.semitones() as i32;
    let pc = (pitch as i32 - root_pc).rem_euclid(12);
    if mask & (1 << pc) != 0 {
        return pitch;
    }
    let mut best_delta: i32 = i32::MAX;
    for candidate in 0..12_i32 {
        if mask & (1 << candidate) == 0 {
            continue;
        }
        let mut d = candidate - pc;
        if d > 6 {
            d -= 12;
        } else if d < -6 {
            d += 12;
        }
        // Prefer smaller |d|; on exact ties (|d| == |best|), prefer the lower
        // neighbor (more negative d) — fewer accidentals when snapping in.
        if d.abs() < best_delta.abs()
            || (d.abs() == best_delta.abs() && d < best_delta)
        {
            best_delta = d;
        }
    }
    (pitch as i32 + best_delta).clamp(0, 127) as u8
}

const MAX_HELD: usize = 16;

#[derive(Clone, Copy)]
struct HeldNote {
    note: u8,
    velocity: f32,
}

pub struct Arpeggiator {
    held: Vec<HeldNote>,
    step_idx: i64,
    current_playing: Option<u8>,
    gate_samples_remaining: i32,
    last_host_step: Option<i64>,
    free_phase: f32,
    sample_rate: f32,
    rng_state: u32,
}

#[derive(Default)]
pub struct ArpTick {
    pub note_off: Option<u8>,
    pub note_on: Option<(u8, f32)>,
}

impl Arpeggiator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            held: Vec::with_capacity(MAX_HELD),
            step_idx: 0,
            current_playing: None,
            gate_samples_remaining: 0,
            last_host_step: None,
            free_phase: 0.0,
            sample_rate,
            rng_state: 0xDEAD_BEEF,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
        self.free_phase = 0.0;
    }

    pub fn reset(&mut self) {
        self.held.clear();
        self.step_idx = 0;
        self.current_playing = None;
        self.gate_samples_remaining = 0;
        self.last_host_step = None;
        self.free_phase = 0.0;
    }

    /// Called when the arp is turned off (or the plugin is reset) to emit
    /// a final note_off so no voice gets stuck.
    pub fn flush(&mut self) -> Option<u8> {
        self.gate_samples_remaining = 0;
        self.last_host_step = None;
        self.free_phase = 0.0;
        self.current_playing.take()
    }

    pub fn add_held(&mut self, note: u8, velocity: f32) {
        if self.held.len() >= MAX_HELD {
            return;
        }
        if !self.held.iter().any(|h| h.note == note) {
            self.held.push(HeldNote { note, velocity });
        }
    }

    pub fn remove_held(&mut self, note: u8) {
        self.held.retain(|h| h.note != note);
    }

    pub fn has_notes(&self) -> bool {
        !self.held.is_empty() || self.current_playing.is_some()
    }

    fn next_rand(&mut self) -> u32 {
        // xorshift32
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng_state = x;
        x
    }

    fn pick_note(
        &mut self,
        pattern: ArpPattern,
        octaves: u8,
        scale: ArpScale,
        root: ArpRoot,
    ) -> Option<(u8, f32)> {
        let octaves = octaves.max(1) as i32;
        if self.held.is_empty() {
            return None;
        }

        let mut ordered: Vec<HeldNote> = self.held.clone();
        match pattern {
            ArpPattern::AsPlayed | ArpPattern::Random => {}
            _ => ordered.sort_by_key(|h| h.note),
        }

        let base_len = ordered.len() as i32;
        let total = base_len * octaves;

        let pos = match pattern {
            ArpPattern::Up | ArpPattern::AsPlayed => {
                let p = self.step_idx.rem_euclid(total as i64) as i32;
                self.step_idx = self.step_idx.wrapping_add(1);
                p
            }
            ArpPattern::Down => {
                let p = (total - 1) - (self.step_idx.rem_euclid(total as i64) as i32);
                self.step_idx = self.step_idx.wrapping_add(1);
                p
            }
            ArpPattern::UpDown => {
                self.step_idx = self.step_idx.wrapping_add(1);
                if total <= 1 {
                    0
                } else {
                    let period = (2 * (total - 1)) as i64;
                    let m = (self.step_idx - 1).rem_euclid(period) as i32;
                    if m < total { m } else { (2 * (total - 1)) - m }
                }
            }
            ArpPattern::Random => {
                self.step_idx = self.step_idx.wrapping_add(1);
                (self.next_rand() % (total as u32)) as i32
            }
        };

        let oct = pos / base_len;
        let note_idx = (pos % base_len) as usize;
        let h = ordered[note_idx];
        let raw = (h.note as i32 + oct * 12).clamp(0, 127) as u8;
        Some((snap_to_scale(raw, scale, root), h.velocity))
    }

    /// Advance one sample. Returns any note_on / note_off events to apply.
    pub fn tick(
        &mut self,
        host_beats: Option<f64>,
        beats_per_step: f32,
        bpm: f32,
        pattern: ArpPattern,
        octaves: u8,
        scale: ArpScale,
        root: ArpRoot,
        gate: f32,
    ) -> ArpTick {
        let mut out = ArpTick::default();

        let samples_per_step =
            (60.0 / bpm.max(1.0) * beats_per_step.max(1e-4) * self.sample_rate).max(1.0);
        let gate_samples = (gate.clamp(0.05, 0.95) * samples_per_step) as i32;

        let step_tick = match host_beats {
            Some(b) => {
                let idx = (b / beats_per_step as f64).floor() as i64;
                match self.last_host_step {
                    None => {
                        self.last_host_step = Some(idx);
                        true
                    }
                    Some(prev) if idx != prev => {
                        self.last_host_step = Some(idx);
                        true
                    }
                    _ => false,
                }
            }
            None => {
                self.free_phase += 1.0 / samples_per_step;
                if self.free_phase >= 1.0 {
                    self.free_phase -= 1.0;
                    true
                } else {
                    false
                }
            }
        };

        // Gate timer for the currently-playing note.
        if self.current_playing.is_some() {
            self.gate_samples_remaining -= 1;
            if self.gate_samples_remaining <= 0 {
                if let Some(note) = self.current_playing.take() {
                    out.note_off = Some(note);
                }
            }
        }

        if step_tick && !self.held.is_empty() {
            if let Some(note) = self.current_playing.take() {
                out.note_off = Some(note);
            }
            if let Some((n, v)) = self.pick_note(pattern, octaves, scale, root) {
                self.current_playing = Some(n);
                self.gate_samples_remaining = gate_samples.max(1);
                out.note_on = Some((n, v));
            }
        }

        out
    }
}
