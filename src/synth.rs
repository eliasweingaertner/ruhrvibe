//! Main plugin implementation.
//!
//! Implements the `Plugin`, `ClapPlugin`, and `Vst3Plugin` traits from
//! nih-plug. Manages the voice pool, handles MIDI events, and runs the
//! per-sample processing loop. Parameters are read from smoothers once
//! per sample and passed as scalar values to each voice.

use nih_plug::prelude::*;
use std::sync::Arc;

use crate::fx::chorus::Chorus;
use crate::fx::delay::Delay;
use crate::params::{FilterParams, OscParams, SynthParams};
use crate::voice::{
    EnvelopeVoiceParams, FilterVoiceParams, OscBankPrecomp, OscVoiceParams,
    PitchEnvVoiceParams, Voice, VoiceParams,
};

/// Maximum number of polyphonic voices (pre-allocated pool).
const MAX_VOICES: usize = 32;

pub struct SubtractiveSynth {
    params: Arc<SynthParams>,
    voices: Vec<Voice>,
    chorus: Chorus,
    delay: Delay,
    sample_rate: f32,
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
            sample_rate,
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

    #[inline]
    fn osc_voice_params(params: &OscParams) -> OscVoiceParams {
        OscVoiceParams {
            waveform: params.waveform.value(),
            level: params.level.smoothed.next(),
            detune_cents: params.detune.smoothed.next(),
            octave: params.octave.value(),
            enabled: params.enabled.value(),
            unison_voices: params.unison_voices.value(),
            unison_spread: params.unison_spread.smoothed.next(),
            pan: params.pan.smoothed.next(),
            stereo_spread: params.stereo_spread.smoothed.next(),
        }
    }

    #[inline]
    fn filter_voice_params(params: &FilterParams) -> FilterVoiceParams {
        FilterVoiceParams {
            filter_type: params.filter_type.value(),
            cutoff: params.cutoff.smoothed.next(),
            resonance: params.resonance.smoothed.next(),
            drive: params.drive.smoothed.next(),
            env_amount: params.env_amount.smoothed.next(),
            enabled: params.enabled.value(),
        }
    }

    #[inline]
    fn env_voice_params(params: &crate::params::EnvelopeParams) -> EnvelopeVoiceParams {
        EnvelopeVoiceParams {
            attack: params.attack.smoothed.next(),
            decay: params.decay.smoothed.next(),
            sustain: params.sustain.smoothed.next(),
            release: params.release.smoothed.next(),
        }
    }

    #[inline]
    fn pitch_env_voice_params(params: &crate::params::PitchEnvParams) -> PitchEnvVoiceParams {
        PitchEnvVoiceParams {
            attack: params.attack.smoothed.next(),
            decay: params.decay.smoothed.next(),
            sustain: params.sustain.smoothed.next(),
            release: params.release.smoothed.next(),
            amount: params.amount.smoothed.next(),
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
        for voice in &mut self.voices {
            voice.set_sample_rate(self.sample_rate);
            voice.reset();
        }
        self.chorus.set_sample_rate(self.sample_rate);
        self.delay.set_sample_rate(self.sample_rate);
        true
    }

    fn reset(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }
        self.chorus.reset();
        self.delay.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let max_voices = (self.params.num_voices.value() as usize).min(MAX_VOICES);
        let chorus_enabled = self.params.chorus.enabled.value();
        let delay_enabled = self.params.delay.enabled.value();
        let fx_active = chorus_enabled || delay_enabled;

        let mut next_event = context.next_event();
        let mut any_active = self.active_voice_count(max_voices) > 0 || next_event.is_some();

        for (sample_id, channel_samples) in buffer.iter_samples().enumerate() {
            // Drain any MIDI events that should fire at this sample.
            while let Some(event) = next_event {
                if event.timing() as usize > sample_id {
                    break;
                }
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        self.note_on(note, velocity);
                        any_active = true;
                    }
                    NoteEvent::NoteOff { note, .. } => {
                        self.note_off(note);
                    }
                    NoteEvent::Choke { note, .. } => {
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

            // Fast path: skip everything only when voices are silent AND no
            // FX are active. Delay feedback needs the chain running even on
            // zero input so tails decay naturally.
            if !any_active && !fx_active {
                for sample in channel_samples {
                    *sample = 0.0;
                }
                continue;
            }

            // Voice mix.
            let (mut mix_l, mut mix_r) = (0.0f32, 0.0f32);
            let master_gain = self.params.master_gain.smoothed.next();
            if any_active {
                let osc1 = Self::osc_voice_params(&self.params.osc1);
                let osc2 = Self::osc_voice_params(&self.params.osc2);
                let osc1_pre = OscBankPrecomp::compute(&osc1);
                let osc2_pre = OscBankPrecomp::compute(&osc2);
                let voice_params = VoiceParams {
                    osc1,
                    osc2,
                    osc1_pre,
                    osc2_pre,
                    filter1: Self::filter_voice_params(&self.params.filter1),
                    filter2: Self::filter_voice_params(&self.params.filter2),
                    amp_env: Self::env_voice_params(&self.params.amp_env),
                    filter1_env: Self::env_voice_params(&self.params.filter1_env),
                    filter2_env: Self::env_voice_params(&self.params.filter2_env),
                    pitch_env: Self::pitch_env_voice_params(&self.params.pitch_env),
                };

                let mut found_active = false;
                for voice in self.voices.iter_mut().take(max_voices) {
                    if voice.is_active() {
                        let (l, r) = voice.process(&voice_params);
                        mix_l += l;
                        mix_r += r;
                        found_active = true;
                    }
                }
                mix_l *= master_gain;
                mix_r *= master_gain;

                if !found_active {
                    any_active = false;
                }
            }

            // FX chain (chorus → delay). Always runs its smoothers and DSP
            // when enabled so tails continue after voices release.
            if chorus_enabled {
                let rate = self.params.chorus.rate.smoothed.next();
                let depth = self.params.chorus.depth.smoothed.next();
                let mix = self.params.chorus.mix.smoothed.next();
                let (l, r) = self.chorus.process(mix_l, mix_r, rate, depth, mix);
                mix_l = l;
                mix_r = r;
            }
            if delay_enabled {
                let time_ms = self.params.delay.time_ms.smoothed.next();
                let feedback = self.params.delay.feedback.smoothed.next();
                let tone = self.params.delay.tone.smoothed.next();
                let mix = self.params.delay.mix.smoothed.next();
                let (l, r) = self.delay.process(mix_l, mix_r, time_ms, feedback, tone, mix);
                mix_l = l;
                mix_r = r;
            }

            for (ch, sample) in channel_samples.into_iter().enumerate() {
                *sample = if ch == 0 { mix_l } else { mix_r };
            }
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
