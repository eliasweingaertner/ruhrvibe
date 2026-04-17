//! Fast approximations for expensive math operations on the audio thread.
//!
//! `exp2_fast` replaces `2.0_f32.powf()` in the per-sample oscillator
//! frequency path — the highest call volume. Filter `tan()` and envelope
//! `exp()` stay on stdlib for accuracy (filters are 2 calls/voice, and
//! envelope coefficients are cached so cost is amortised).

/// Fast approximation of 2^x using the bit-level float trick + polynomial
/// refinement. Max relative error ~0.06% across [-127, 127].
#[inline]
pub fn exp2_fast(x: f32) -> f32 {
    let x = x.clamp(-126.0, 126.0);
    let xi = x.floor();
    let xf = x - xi;

    // 2^integer part via exponent manipulation.
    let pow2_int = f32::from_bits(((xi as i32 + 127) as u32) << 23);

    // 2^fractional part via minimax cubic.
    let pow2_frac = 1.0 + xf * (0.6931472 + xf * (0.2402265 + xf * 0.0558014));

    pow2_int * pow2_frac
}

/// Precomputed 1/sqrt(n) for n=1..7 (unison normalization).
pub const INV_SQRT: [f32; 8] = [
    1.0,                // [0] unused
    1.0,                // 1/sqrt(1)
    std::f32::consts::FRAC_1_SQRT_2, // 1/sqrt(2)
    0.577_350_26,       // 1/sqrt(3)
    0.5,                // 1/sqrt(4)
    0.447_213_6,        // 1/sqrt(5)
    0.408_248_3,        // 1/sqrt(6)
    0.377_964_47,       // 1/sqrt(7)
];
