//! Main plugin implementation.
//!
//! Implements the `Plugin`, `ClapPlugin`, and `Vst3Plugin` traits from
//! nih-plug. Manages the voice pool, handles MIDI events, and runs the
//! per-sample processing loop. Parameters are read from smoothers once
//! per sample and passed as scalar values to each voice.

use nih_plug::prelude::*;
use std::f32::consts::PI;
use std::sync::Arc;

use crate::arp::Arpeggiator;
use crate::filter::SvfCoeffs;
use crate::fx::chorus::Chorus;
use crate::fx::delay::Delay;
use crate::fx::distortion::Distortion;
use crate::fx::gapper::Gapper;
use crate::fx::reverb::Reverb;
use crate::fx::shimmer::Shimmer;
use crate::params::SynthParams;
use crate::voice::{
    EnvelopeVoiceParams, FilterVoiceParams, OscBankPrecomp, OscVoiceParams,
    PitchEnvVoiceParams, Voice, VoiceParams,
};

/// Maximum number of polyphonic voices (pre-allocated pool).
const MAX_VOICES: usize = 32;

/// Sub-block size for block-based processing. Parameters are read once per
/// sub-block (advancing smoothers by this many steps), then shared across all
/// voices for the block. MIDI events split sub-blocks at sample-accurate
/// boundaries — no timing degradation, at most ~1.45 ms arp quantisation.
pub const BLOCK_SIZE: usize = 64;

pub struct SubtractiveSynth {
    params: Arc<SynthParams>,
    voices: Vec<Voice>,
    chorus: Chorus,
    delay: Delay,
    shimmer: Shimmer,
    gapper: Gapper,
    reverb: Reverb,
    distortion: Distortion,
    arp: Arpeggiator,
    /// Whether the arp was enabled on the previous sample — used to detect
    /// disable transitions so a final note_off can flush any held voice.
    arp_was_enabled: bool,
    sample_rate: f32,
    /// `π / sample_rate`, recomputed on sample-rate change and handed to
    /// voices so SVF coefficient recomputation is `tan(pi_over_fs * fc)`
    /// instead of `tan(pi * fc / fs)`.
    pi_over_fs: f32,
    /// Upper cutoff limit (~Nyquist) — `0.49 * sample_rate`.
    nyquist: f32,
    /// Cached `OscBankPrecomp` + the source params it was computed from.
    /// `OscBankPrecomp::compute` runs several `exp2_fast` and trig calls per
    /// unison voice; caching lets the common "smoothers idle" case reuse the
    /// previous sample's result.
    cached_osc1_params: Option<OscVoiceParams>,
    cached_osc1_pre: Option<OscBankPrecomp>,
    cached_osc2_params: Option<OscVoiceParams>,
    cached_osc2_pre: Option<OscBankPrecomp>,
}

impl Default for SubtractiveSynth {
    fn default() -> Self {
        let sample_rate = 44_100.0;
        let voices = (0..MAX_VOICES).map(|_| Voice::new(sample_rate)).collect();
        Self {
            params: Arc::new(SynthParams::default()),
            voices,
            chorus: Chorus::new(sample_rate),
            delay: Delay::new(sample_rate),
            shimmer: Shimmer::new(sample_rate),
            gapper: Gapper::new(sample_rate),
            reverb: Reverb::new(sample_rate),
            distortion: Distortion::new(),
            arp: Arpeggiator::new(sample_rate),
            arp_was_enabled: false,
            sample_rate,
            pi_over_fs: PI / sample_rate,
            nyquist: 0.49 * sample_rate,
            cached_osc1_params: None,
            cached_osc1_pre: None,
            cached_osc2_params: None,
            cached_osc2_pre: None,
        }
    }
}

impl SubtractiveSynth {
    fn note_on(&mut self, note: u8, velocity: f32) {
        let max_voices = self.params.num_voices.value() as usize;
        let active_pool = &mut self.voices[..max_voices.min(MAX_VOICES)];

        if let Some(voice) = active_pool.iter_mut().find(|v| !v.is_active()) {
            voice.note_on(note, velocity);
            return;
        }

        if let Some(voice) = active_pool
            .iter_mut()
            .min_by(|a, b| a.amp_level().partial_cmp(&b.amp_level()).unwrap_or(std::cmp::Ordering::Equal))
        {
            voice.note_on(note, velocity);
        }
    }

    fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.is_active() && voice.note == note {
                voice.note_off();
            }
        }
    }

    /// Build a `VoiceParams` snapshot for one sub-block.
    ///
    /// Each `FloatParam` smoother is advanced by exactly `block_len` steps via
    /// `next_block`, keeping smoother timing identical to per-sample reads.
    /// A single 64-element stack buffer is reused across all reads — only the
    /// first value (`tmp[0]`) is used as the representative for the block.
    fn build_voice_params(&mut self, block_len: usize) -> VoiceParams {
        let mut tmp = [0.0f32; BLOCK_SIZE];

        macro_rules! rd {
            ($s:expr) => {{
                $s.next_block(&mut tmp, block_len);
                tmp[0]
            }};
        }

        let osc1 = OscVoiceParams {
            waveform:       self.params.osc1.waveform.value(),
            level:          rd!(self.params.osc1.level.smoothed),
            detune_cents:   rd!(self.params.osc1.detune.smoothed),
            octave:         self.params.osc1.octave.value(),
            enabled:        self.params.osc1.enabled.value(),
            unison_voices:  self.params.osc1.unison_voices.value(),
            unison_spread:  rd!(self.params.osc1.unison_spread.smoothed),
            pan:            rd!(self.params.osc1.pan.smoothed),
            stereo_spread:  rd!(self.params.osc1.stereo_spread.smoothed),
        };
        let osc2 = OscVoiceParams {
            waveform:       self.params.osc2.waveform.value(),
            level:          rd!(self.params.osc2.level.smoothed),
            detune_cents:   rd!(self.params.osc2.detune.smoothed),
            octave:         self.params.osc2.octave.value(),
            enabled:        self.params.osc2.enabled.value(),
            unison_voices:  self.params.osc2.unison_voices.value(),
            unison_spread:  rd!(self.params.osc2.unison_spread.smoothed),
            pan:            rd!(self.params.osc2.pan.smoothed),
            stereo_spread:  rd!(self.params.osc2.stereo_spread.smoothed),
        };

        let osc1_pre = match self.cached_osc1_pre {
            Some(pre) if self.cached_osc1_params == Some(osc1) => pre,
            _ => {
                let pre = OscBankPrecomp::compute(&osc1);
                self.cached_osc1_params = Some(osc1);
                self.cached_osc1_pre   = Some(pre);
                pre
            }
        };
        let osc2_pre = match self.cached_osc2_pre {
            Some(pre) if self.cached_osc2_params == Some(osc2) => pre,
            _ => {
                let pre = OscBankPrecomp::compute(&osc2);
                self.cached_osc2_params = Some(osc2);
                self.cached_osc2_pre   = Some(pre);
                pre
            }
        };

        let filter1 = FilterVoiceParams {
            filter_type: self.params.filter1.filter_type.value(),
            cutoff:      rd!(self.params.filter1.cutoff.smoothed),
            resonance:   rd!(self.params.filter1.resonance.smoothed),
            drive:       rd!(self.params.filter1.drive.smoothed),
            env_amount:  rd!(self.params.filter1.env_amount.smoothed),
            enabled:     self.params.filter1.enabled.value(),
        };
        let filter2 = FilterVoiceParams {
            filter_type: self.params.filter2.filter_type.value(),
            cutoff:      rd!(self.params.filter2.cutoff.smoothed),
            resonance:   rd!(self.params.filter2.resonance.smoothed),
            drive:       rd!(self.params.filter2.drive.smoothed),
            env_amount:  rd!(self.params.filter2.env_amount.smoothed),
            enabled:     self.params.filter2.enabled.value(),
        };

        let filter1_coeffs = if filter1.enabled && filter1.env_amount == 0.0 {
            Some(SvfCoeffs::compute(
                filter1.cutoff, filter1.resonance, filter1.drive,
                filter1.filter_type, self.pi_over_fs, self.nyquist,
            ))
        } else { None };
        let filter2_coeffs = if filter2.enabled && filter2.env_amount == 0.0 {
            Some(SvfCoeffs::compute(
                filter2.cutoff, filter2.resonance, filter2.drive,
                filter2.filter_type, self.pi_over_fs, self.nyquist,
            ))
        } else { None };

        VoiceParams {
            osc1, osc2, osc1_pre, osc2_pre,
            filter1, filter2, filter1_coeffs, filter2_coeffs,
            amp_env: EnvelopeVoiceParams {
                attack:  rd!(self.params.amp_env.attack.smoothed),
                decay:   rd!(self.params.amp_env.decay.smoothed),
                sustain: rd!(self.params.amp_env.sustain.smoothed),
                release: rd!(self.params.amp_env.release.smoothed),
            },
            filter1_env: EnvelopeVoiceParams {
                attack:  rd!(self.params.filter1_env.attack.smoothed),
                decay:   rd!(self.params.filter1_env.decay.smoothed),
                sustain: rd!(self.params.filter1_env.sustain.smoothed),
                release: rd!(self.params.filter1_env.release.smoothed),
            },
            filter2_env: EnvelopeVoiceParams {
                attack:  rd!(self.params.filter2_env.attack.smoothed),
                decay:   rd!(self.params.filter2_env.decay.smoothed),
                sustain: rd!(self.params.filter2_env.sustain.smoothed),
                release: rd!(self.params.filter2_env.release.smoothed),
            },
            pitch_env: PitchEnvVoiceParams {
                attack:  rd!(self.params.pitch_env.attack.smoothed),
                decay:   rd!(self.params.pitch_env.decay.smoothed),
                sustain: rd!(self.params.pitch_env.sustain.smoothed),
                release: rd!(self.params.pitch_env.release.smoothed),
                amount:  rd!(self.params.pitch_env.amount.smoothed),
            },
            pi_over_fs: self.pi_over_fs,
            nyquist:    self.nyquist,
            osc1_fm_amount: rd!(self.params.osc1.fm_amount.smoothed),
        }
    }

    /// Count how many voices are currently active in the pool.
    #[inline]
    fn active_voice_count(&self, max_voices: usize) -> usize {
        self.voices.iter().take(max_voices).filter(|v| v.is_active()).count()
    }
}

impl Plugin for SubtractiveSynth {
    const NAME: &'static str = "Ruhrvibe";
    const VENDOR: &'static str = "Ruhrvibe";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: std::num::NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        crate::editor::create(self.params.clone(), self.params.editor_state.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        self.pi_over_fs = PI / self.sample_rate;
        self.nyquist = 0.49 * self.sample_rate;
        for voice in &mut self.voices {
            voice.set_sample_rate(self.sample_rate);
            voice.reset();
        }
        self.chorus.set_sample_rate(self.sample_rate);
        self.delay.set_sample_rate(self.sample_rate);
        self.shimmer.set_sample_rate(self.sample_rate);
        self.gapper.set_sample_rate(self.sample_rate);
        self.reverb.set_sample_rate(self.sample_rate);
        self.arp.set_sample_rate(self.sample_rate);
        self.cached_osc1_params = None;
        self.cached_osc1_pre = None;
        self.cached_osc2_params = None;
        self.cached_osc2_pre = None;
        true
    }

    fn reset(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }
        self.chorus.reset();
        self.delay.reset();
        self.shimmer.reset();
        self.gapper.reset();
        self.reverb.reset();
        self.distortion.reset();
        self.arp.reset();
        self.arp_was_enabled = false;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let max_voices = (self.params.num_voices.value() as usize).min(MAX_VOICES);
        let chorus_enabled     = self.params.chorus.pos.value()     > 0;
        let delay_enabled      = self.params.delay.pos.value()      > 0;
        let shimmer_enabled    = self.params.shimmer.pos.value()    > 0;
        let gapper_enabled     = self.params.gapper.pos.value()     > 0;
        let reverb_enabled     = self.params.reverb.pos.value()     > 0;
        let distortion_enabled = self.params.distortion.pos.value() > 0;
        let fx_active = chorus_enabled || delay_enabled || shimmer_enabled
            || gapper_enabled || reverb_enabled || distortion_enabled;

        // Build ordered FX list once per DAW buffer.
        // Each entry: (chain position, fx_id, name for tiebreak).
        // fx_id: 0=Chorus, 1=Delay, 2=Shimmer, 3=Gapper, 4=Reverb, 5=Distortion
        // Position 0 means Off — excluded. Equal positions → lexicographic name order.
        let mut raw_fx = [
            (self.params.chorus.pos.value(),     0u8, "Chorus"),
            (self.params.delay.pos.value(),      1u8, "Delay"),
            (self.params.shimmer.pos.value(),    2u8, "Shimmer"),
            (self.params.gapper.pos.value(),     3u8, "Gapper"),
            (self.params.reverb.pos.value(),     4u8, "Reverb"),
            (self.params.distortion.pos.value(), 5u8, "Distortion"),
        ];
        raw_fx.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.2.cmp(b.2)));
        let mut sorted_fx_arr = [0u8; 6];
        let mut n_fx = 0usize;
        for &(pos, id, _) in &raw_fx {
            if pos > 0 {
                sorted_fx_arr[n_fx] = id;
                n_fx += 1;
            }
        }
        let sorted_fx = &sorted_fx_arr[..n_fx];

        // Transport info captured once — stable across the whole DAW buffer.
        let transport = context.transport();
        let tempo_bpm        = transport.tempo.unwrap_or(120.0) as f32;
        let block_start_beats = transport.pos_beats();
        let playing          = transport.playing;
        let beats_per_sample = tempo_bpm as f64 / 60.0 / self.sample_rate as f64;
        let gapper_rate_beats = self.params.gapper.rate.value().beats_per_cycle();

        // Arp config (stable across the DAW buffer).
        let arp_enabled    = self.params.arp.enabled.value();
        let arp_pattern    = self.params.arp.pattern.value();
        let arp_rate_beats = self.params.arp.rate.value().beats_per_cycle();
        let arp_octaves    = self.params.arp.octaves.value() as u8;
        let arp_scale      = self.params.arp.scale.value();
        let arp_root       = self.params.arp.root.value();
        let arp_walk       = self.params.arp.walk.value();
        let arp_step       = self.params.arp.step.value() as u8;

        if self.arp_was_enabled && !arp_enabled {
            if let Some(note) = self.arp.flush() {
                self.note_off(note);
            }
        }
        self.arp_was_enabled = arp_enabled;

        let total_samples = buffer.samples();
        // Raw channel slices — indexed as output[ch][sample].
        let output = buffer.as_slice();

        let mut next_event = context.next_event();
        let mut any_active = self.active_voice_count(max_voices) > 0
            || next_event.is_some()
            || (arp_enabled && self.arp.has_notes());

        // Mix buffers reused across sub-blocks (stack, 512 B total).
        let mut mix_l = [0.0f32; BLOCK_SIZE];
        let mut mix_r = [0.0f32; BLOCK_SIZE];

        let mut pos = 0usize;
        while pos < total_samples {
            // ── Sub-block boundary ─────────────────────────────────────────
            // Split at the next MIDI event so note_on/off land on the exact
            // sample, then cap at BLOCK_SIZE.
            let block_end = {
                let natural_end = (pos + BLOCK_SIZE).min(total_samples);
                match next_event {
                    Some(ev) => {
                        let ev_pos = ev.timing() as usize;
                        if ev_pos > pos && ev_pos < natural_end { ev_pos } else { natural_end }
                    }
                    None => natural_end,
                }
            };
            let block_len = block_end - pos;

            // ── 1. Arp pre-pass ────────────────────────────────────────────
            // Per-sample arp tick so host-beat timing stays accurate.
            // Fires note events into the voice pool before voice processing.
            if arp_enabled {
                for s in pos..block_end {
                    let host_beats = if playing {
                        block_start_beats.map(|b| b + s as f64 * beats_per_sample)
                    } else { None };
                    let gate = self.params.arp.gate.smoothed.next();
                    let tick = self.arp.tick(
                        host_beats, arp_rate_beats, tempo_bpm,
                        arp_pattern, arp_octaves, arp_scale, arp_root,
                        arp_walk, arp_step, gate,
                    );
                    if let Some(n) = tick.note_off { self.note_off(n); }
                    if let Some((n, v)) = tick.note_on {
                        self.note_on(n, v);
                        any_active = true;
                    }
                }
            }

            // ── 2. MIDI drain ──────────────────────────────────────────────
            // Consume all events whose timing falls before block_end.
            while let Some(event) = next_event {
                if event.timing() as usize >= block_end { break; }
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        if arp_enabled { self.arp.add_held(note, velocity); }
                        else { self.note_on(note, velocity); }
                        any_active = true;
                    }
                    NoteEvent::NoteOff { note, .. } => {
                        if arp_enabled { self.arp.remove_held(note); }
                        else { self.note_off(note); }
                    }
                    NoteEvent::Choke { note, .. } => {
                        if arp_enabled { self.arp.remove_held(note); }
                        for voice in &mut self.voices {
                            if voice.is_active() && voice.note == note {
                                voice.reset();
                            }
                        }
                    }
                    _ => {}
                }
                next_event = context.next_event();
            }

            // ── Fast path ──────────────────────────────────────────────────
            if !any_active && !fx_active {
                for i in 0..block_len {
                    output[0][pos + i] = 0.0;
                    output[1][pos + i] = 0.0;
                }
                pos = block_end;
                continue;
            }

            // ── 3. Voice processing ────────────────────────────────────────
            // VoiceParams built ONCE per sub-block; all smoothers advance by
            // block_len steps so timing is equivalent to per-sample reads.
            for i in 0..block_len { mix_l[i] = 0.0; mix_r[i] = 0.0; }

            // master_gain smoother also advances by block_len steps.
            let master_gain = {
                let mut g = [0.0f32; BLOCK_SIZE];
                self.params.master_gain.smoothed.next_block(&mut g, block_len);
                g[0]
            };

            if any_active {
                let voice_params = self.build_voice_params(block_len);
                let mut found_active = false;
                for voice in self.voices.iter_mut().take(max_voices) {
                    if voice.is_active() {
                        voice.process_block(
                            &voice_params,
                            &mut mix_l[..block_len],
                            &mut mix_r[..block_len],
                        );
                        found_active = true;
                    }
                }
                if !found_active { any_active = false; }
                for i in 0..block_len {
                    mix_l[i] *= master_gain;
                    mix_r[i] *= master_gain;
                }
            }

            // ── 4. FX chain ────────────────────────────────────────────────
            // Params read once per sub-block (smoothers advance by block_len).
            // Gapper needs per-sample host_beats for sync; other FX use the
            // block-start value which is indistinguishable at 64-sample blocks.
            if fx_active {
                let mut ft = [0.0f32; BLOCK_SIZE];
                macro_rules! rfx {
                    ($s:expr) => {{ $s.next_block(&mut ft, block_len); ft[0] }};
                }

                let (ch_rate, ch_depth, ch_mix) = if chorus_enabled {
                    (rfx!(self.params.chorus.rate.smoothed),
                     rfx!(self.params.chorus.depth.smoothed),
                     rfx!(self.params.chorus.mix.smoothed))
                } else { (0.0, 0.0, 0.0) };

                let (dl_time, dl_fb, dl_tone, dl_mix) = if delay_enabled {
                    (rfx!(self.params.delay.time_ms.smoothed),
                     rfx!(self.params.delay.feedback.smoothed),
                     rfx!(self.params.delay.tone.smoothed),
                     rfx!(self.params.delay.mix.smoothed))
                } else { (0.0, 0.0, 0.0, 0.0) };

                let (sh_time, sh_fb, sh_mix) = if shimmer_enabled {
                    (rfx!(self.params.shimmer.time_ms.smoothed),
                     rfx!(self.params.shimmer.feedback.smoothed),
                     rfx!(self.params.shimmer.mix.smoothed))
                } else { (0.0, 0.0, 0.0) };

                let (gp_duty, gp_smooth, gp_depth) = if gapper_enabled {
                    (rfx!(self.params.gapper.duty.smoothed),
                     rfx!(self.params.gapper.smooth.smoothed),
                     rfx!(self.params.gapper.depth.smoothed))
                } else { (0.0, 0.0, 0.0) };

                let (rv_size, rv_damp, rv_width, rv_mix) = if reverb_enabled {
                    (rfx!(self.params.reverb.room_size.smoothed),
                     rfx!(self.params.reverb.damping.smoothed),
                     rfx!(self.params.reverb.width.smoothed),
                     rfx!(self.params.reverb.mix.smoothed))
                } else { (0.0, 0.0, 0.0, 0.0) };

                let (dst_drive, dst_tone, dst_mix) = if distortion_enabled {
                    (rfx!(self.params.distortion.drive.smoothed),
                     rfx!(self.params.distortion.tone.smoothed),
                     rfx!(self.params.distortion.mix.smoothed))
                } else { (0.0, 0.0, 0.0) };
                let dst_type = self.params.distortion.dist_type.value();

                for i in 0..block_len {
                    let (mut l, mut r) = (mix_l[i], mix_r[i]);
                    for &fx_id in sorted_fx {
                        match fx_id {
                            0 if chorus_enabled => {
                                (l, r) = self.chorus.process(l, r, ch_rate, ch_depth, ch_mix);
                            }
                            1 if delay_enabled => {
                                (l, r) = self.delay.process(l, r, dl_time, dl_fb, dl_tone, dl_mix);
                            }
                            2 if shimmer_enabled => {
                                (l, r) = self.shimmer.process(l, r, sh_time, sh_fb, sh_mix);
                            }
                            3 if gapper_enabled => {
                                let host_beats = if playing {
                                    block_start_beats.map(|b| b + (pos + i) as f64 * beats_per_sample)
                                } else { None };
                                (l, r) = self.gapper.process(
                                    l, r, host_beats, gapper_rate_beats,
                                    tempo_bpm, gp_duty, gp_smooth, gp_depth,
                                );
                            }
                            4 if reverb_enabled => {
                                (l, r) = self.reverb.process(
                                    l, r, rv_size, rv_damp, rv_width, rv_mix,
                                );
                            }
                            5 if distortion_enabled => {
                                (l, r) = self.distortion.process(
                                    l, r, dst_drive, dst_type, dst_tone, dst_mix,
                                );
                            }
                            _ => {}
                        }
                    }
                    mix_l[i] = l;
                    mix_r[i] = r;
                }
            }

            // ── 5. Write to output ─────────────────────────────────────────
            for i in 0..block_len {
                output[0][pos + i] = mix_l[i];
                output[1][pos + i] = mix_r[i];
            }

            pos = block_end;
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for SubtractiveSynth {
    const CLAP_ID: &'static str = "com.ruhrvibe.synth";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Ruhrvibe - a polyphonic subtractive synthesizer");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for SubtractiveSynth {
    const VST3_CLASS_ID: [u8; 16] = *b"RuhrvibeSynth_00";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}
