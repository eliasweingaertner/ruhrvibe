//! Waveshaping distortion with 1-pole tone filter.

use crate::params::DistType;

pub struct Distortion {
    lp_l: f32,
    lp_r: f32,
}

impl Distortion {
    pub fn new() -> Self { Self { lp_l: 0.0, lp_r: 0.0 } }

    pub fn reset(&mut self) { self.lp_l = 0.0; self.lp_r = 0.0; }

    #[inline]
    pub fn process(
        &mut self,
        in_l: f32,
        in_r: f32,
        drive: f32,
        dist_type: DistType,
        tone: f32,
        mix: f32,
    ) -> (f32, f32) {
        // Partial gain compensation: 0.25-power so high drive stays loud and brutal.
        // (0.5-power was too quiet at extreme settings.)
        let comp = 1.0 / drive.max(1.0).powf(0.25);
        let shaped_l = Self::shape(in_l * drive, dist_type) * comp;
        let shaped_r = Self::shape(in_r * drive, dist_type) * comp;

        // 1-pole LP tone filter (a=0 → bright/no filter, a→1 → dark).
        let a = (1.0 - tone).clamp(0.0, 0.97);
        self.lp_l += (shaped_l - self.lp_l) * (1.0 - a);
        self.lp_r += (shaped_r - self.lp_r) * (1.0 - a);

        let dry = 1.0 - mix;
        (in_l * dry + self.lp_l * mix, in_r * dry + self.lp_r * mix)
    }

    #[inline]
    fn shape(x: f32, dist_type: DistType) -> f32 {
        match dist_type {
            // Smooth tanh saturation.
            DistType::Soft => x.tanh(),
            // Hard brick-wall clip.
            DistType::Hard => x.clamp(-1.0, 1.0),
            // Savage fuzz: triple tanh cascade for maximum harmonic density.
            DistType::Fuzz => (x * 4.0).tanh().tanh() * 0.75,
            // Algebraic soft clip — gentle, tube-like.
            DistType::Warm => x / (1.0 + x.abs()),
        }
    }
}
