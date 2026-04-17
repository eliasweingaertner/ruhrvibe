//! ADSR envelope with exponential curves.
//!
//! Each stage uses `level += (target - level) * coeff` style exponential
//! approach. Coefficients are cached and only recomputed when the time
//! parameter changes, avoiding per-sample `exp()` calls.


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Envelope {
    stage: EnvelopeStage,
    level: f32,
    sample_rate: f32,
    // Cached coefficients + the time values they were computed from.
    cached_attack_time: f32,
    cached_attack_coeff: f32,
    cached_decay_time: f32,
    cached_decay_coeff: f32,
    cached_release_time: f32,
    cached_release_coeff: f32,
}

impl Envelope {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            stage: EnvelopeStage::Idle,
            level: 0.0,
            sample_rate,
            cached_attack_time: -1.0,
            cached_attack_coeff: 1.0,
            cached_decay_time: -1.0,
            cached_decay_coeff: 1.0,
            cached_release_time: -1.0,
            cached_release_coeff: 1.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        // Invalidate caches.
        self.cached_attack_time = -1.0;
        self.cached_decay_time = -1.0;
        self.cached_release_time = -1.0;
    }

    pub fn trigger(&mut self) {
        self.stage = EnvelopeStage::Attack;
    }

    pub fn release(&mut self) {
        if self.stage != EnvelopeStage::Idle {
            self.stage = EnvelopeStage::Release;
        }
    }

    pub fn reset(&mut self) {
        self.stage = EnvelopeStage::Idle;
        self.level = 0.0;
    }

    #[inline]
    pub fn is_idle(&self) -> bool {
        self.stage == EnvelopeStage::Idle
    }

    #[inline]
    pub fn level(&self) -> f32 {
        self.level
    }

    #[inline]
    fn coeff_for_time(&self, time_secs: f32) -> f32 {
        if time_secs <= 0.0 {
            return 1.0;
        }
        1.0 - (-1.0 / (time_secs * self.sample_rate)).exp()
    }

    #[inline]
    fn get_attack_coeff(&mut self, time: f32) -> f32 {
        if time != self.cached_attack_time {
            self.cached_attack_time = time;
            self.cached_attack_coeff = self.coeff_for_time(time);
        }
        self.cached_attack_coeff
    }

    #[inline]
    fn get_decay_coeff(&mut self, time: f32) -> f32 {
        if time != self.cached_decay_time {
            self.cached_decay_time = time;
            self.cached_decay_coeff = self.coeff_for_time(time);
        }
        self.cached_decay_coeff
    }

    #[inline]
    fn get_release_coeff(&mut self, time: f32) -> f32 {
        if time != self.cached_release_time {
            self.cached_release_time = time;
            self.cached_release_coeff = self.coeff_for_time(time);
        }
        self.cached_release_coeff
    }

    /// Advance one sample and return the current level.
    #[inline]
    pub fn next_sample(
        &mut self,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    ) -> f32 {
        match self.stage {
            EnvelopeStage::Idle => {
                self.level = 0.0;
            }
            EnvelopeStage::Attack => {
                let coeff = self.get_attack_coeff(attack);
                self.level += (1.2 - self.level) * coeff;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = EnvelopeStage::Decay;
                }
            }
            EnvelopeStage::Decay => {
                let coeff = self.get_decay_coeff(decay);
                self.level += (sustain - self.level) * coeff;
                if (self.level - sustain).abs() < 1e-4 {
                    self.level = sustain;
                    self.stage = EnvelopeStage::Sustain;
                }
            }
            EnvelopeStage::Sustain => {
                self.level = sustain;
            }
            EnvelopeStage::Release => {
                let coeff = self.get_release_coeff(release);
                self.level -= self.level * coeff;
                if self.level < 1e-4 {
                    self.level = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
            }
        }
        self.level
    }
}
