//! Global effects chain applied after the voice mixer.
//!
//! Effects are single-instance (not per-voice) and share state across the
//! whole plugin. Each effect owns its own delay buffers and LFO state and
//! is driven by per-sample scalar parameter values read from smoothers.

pub mod chorus;
pub mod delay;
pub mod gapper;
pub mod shimmer;
