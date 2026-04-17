//! Iced-based GUI editor for the subtractive synth.
//!
//! HiDPI strategy: rather than rely on baseview/iced scale policies
//! (which are inconsistent across VST3 hosts), we query the system DPI
//! ourselves at editor creation time and scale *both* the reported
//! window size *and* every widget dimension by that factor. The plugin
//! runs Iced at an effective 1x scale — all dimensions are already in
//! physical pixels.

use nih_plug::prelude::*;
use nih_plug_iced::widgets as nih_widgets;
use nih_plug_iced::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use crate::params::SynthParams;
use crate::presets::{apply_preset, FACTORY_PRESETS};

/// Base (logical @96dpi) window size.
const BASE_WIDTH: u32 = 920;
const BASE_HEIGHT: u32 = 780;

/// Cached DPI scale factor × 1000 (to fit in an atomic int).
/// Queried once on the first call to `dpi_scale()`.
static DPI_SCALE_MILLI: AtomicU32 = AtomicU32::new(0);

/// Query (and cache) the system DPI scale factor.
fn dpi_scale() -> f32 {
    let cached = DPI_SCALE_MILLI.load(Ordering::Relaxed);
    if cached != 0 {
        return cached as f32 / 1000.0;
    }
    let factor = query_system_scale_factor();
    DPI_SCALE_MILLI.store((factor * 1000.0) as u32, Ordering::Relaxed);
    factor
}

#[cfg(windows)]
fn query_system_scale_factor() -> f32 {
    use windows_sys::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, ReleaseDC, LOGPIXELSX};
    unsafe {
        let hdc = GetDC(0);
        if hdc == 0 {
            return 1.0;
        }
        let dpi = GetDeviceCaps(hdc, LOGPIXELSX as _);
        ReleaseDC(0, hdc);
        if dpi <= 0 {
            1.0
        } else {
            (dpi as f32 / 96.0).max(1.0)
        }
    }
}

#[cfg(not(windows))]
fn query_system_scale_factor() -> f32 {
    1.0
}

/// Scale a design-time size by the current DPI factor, returning u16 pixels.
#[inline]
fn px(n: f32) -> u16 {
    (n * dpi_scale()).round() as u16
}

pub(crate) fn default_state() -> Arc<IcedState> {
    let scale = dpi_scale();
    IcedState::from_size(
        (BASE_WIDTH as f32 * scale).round() as u32,
        (BASE_HEIGHT as f32 * scale).round() as u32,
    )
}

pub(crate) fn create(
    params: Arc<SynthParams>,
    editor_state: Arc<IcedState>,
) -> Option<Box<dyn Editor>> {
    create_iced_editor::<SynthEditor>(editor_state, params)
}

// ---------------------------------------------------------------------------
// Color palette
// ---------------------------------------------------------------------------

// Palette inspired by the Ruhrvibe logo: dark grey + orange + slate blue.
const COLOR_BG: Color = Color::from_rgb(0.14, 0.15, 0.18);
const COLOR_TEXT: Color = Color::from_rgb(0.92, 0.93, 0.96);
const COLOR_TEXT_DIM: Color = Color::from_rgb(0.60, 0.62, 0.68);

const COLOR_OSC1: Color = Color::from_rgb(0.94, 0.65, 0.15);   // logo orange
const COLOR_OSC2: Color = Color::from_rgb(0.85, 0.55, 0.10);   // darker amber
const COLOR_FLT1: Color = Color::from_rgb(0.25, 0.38, 0.55);   // slate blue (gear)
const COLOR_FLT2: Color = Color::from_rgb(0.35, 0.50, 0.65);   // lighter slate
const COLOR_AMP_ENV: Color = Color::from_rgb(0.94, 0.58, 0.18); // warm orange
const COLOR_F1_ENV: Color = Color::from_rgb(0.30, 0.45, 0.60);  // steel blue
const COLOR_F2_ENV: Color = Color::from_rgb(0.70, 0.52, 0.12);  // bronze
const COLOR_MASTER: Color = Color::from_rgb(0.80, 0.60, 0.15);  // gold
const COLOR_PITCH_ENV: Color = Color::from_rgb(0.50, 0.65, 0.78); // soft blue
const COLOR_HEADER: Color = Color::from_rgb(0.94, 0.65, 0.15);  // logo orange

/// Tint a color towards the dark background, keeping some saturation.
fn tint_bg(color: Color, alpha: f32) -> Color {
    let bg_r = COLOR_BG.r;
    let bg_g = COLOR_BG.g;
    let bg_b = COLOR_BG.b;
    Color::from_rgb(
        bg_r + (color.r - bg_r) * alpha,
        bg_g + (color.g - bg_g) * alpha,
        bg_b + (color.b - bg_b) * alpha,
    )
}

struct SectionStyle {
    accent: Color,
}

impl container::StyleSheet for SectionStyle {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: Some(COLOR_TEXT),
            // Semi-transparent: 15% accent tint over the dark background.
            background: Some(Background::Color(tint_bg(self.accent, 0.15))),
            border_radius: 8.0 * dpi_scale(),
            border_width: 1.5 * dpi_scale(),
            border_color: tint_bg(self.accent, 0.45),
        }
    }
}

struct PanelStyle;

impl container::StyleSheet for PanelStyle {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: Some(COLOR_TEXT),
            background: Some(Background::Color(Color::from_rgba(0.10, 0.11, 0.14, 0.85))),
            border_radius: 10.0 * dpi_scale(),
            border_width: 1.0 * dpi_scale(),
            border_color: Color::from_rgb(0.22, 0.24, 0.28),
        }
    }
}

// ---------------------------------------------------------------------------
// Editor
// ---------------------------------------------------------------------------

struct SynthEditor {
    params: Arc<SynthParams>,
    context: Arc<dyn GuiContext>,

    osc1_wave: nih_widgets::param_slider::State,
    osc1_level: nih_widgets::param_slider::State,
    osc1_detune: nih_widgets::param_slider::State,
    osc1_octave: nih_widgets::param_slider::State,
    osc1_on: nih_widgets::param_slider::State,
    osc1_unison: nih_widgets::param_slider::State,
    osc1_spread: nih_widgets::param_slider::State,

    osc2_wave: nih_widgets::param_slider::State,
    osc2_level: nih_widgets::param_slider::State,
    osc2_detune: nih_widgets::param_slider::State,
    osc2_octave: nih_widgets::param_slider::State,
    osc2_on: nih_widgets::param_slider::State,
    osc2_unison: nih_widgets::param_slider::State,
    osc2_spread: nih_widgets::param_slider::State,

    flt1_type: nih_widgets::param_slider::State,
    flt1_cutoff: nih_widgets::param_slider::State,
    flt1_res: nih_widgets::param_slider::State,
    flt1_drive: nih_widgets::param_slider::State,
    flt1_envamt: nih_widgets::param_slider::State,
    flt1_on: nih_widgets::param_slider::State,

    flt2_type: nih_widgets::param_slider::State,
    flt2_cutoff: nih_widgets::param_slider::State,
    flt2_res: nih_widgets::param_slider::State,
    flt2_drive: nih_widgets::param_slider::State,
    flt2_envamt: nih_widgets::param_slider::State,
    flt2_on: nih_widgets::param_slider::State,

    amp_a: nih_widgets::param_slider::State,
    amp_d: nih_widgets::param_slider::State,
    amp_s: nih_widgets::param_slider::State,
    amp_r: nih_widgets::param_slider::State,

    f1e_a: nih_widgets::param_slider::State,
    f1e_d: nih_widgets::param_slider::State,
    f1e_s: nih_widgets::param_slider::State,
    f1e_r: nih_widgets::param_slider::State,

    f2e_a: nih_widgets::param_slider::State,
    f2e_d: nih_widgets::param_slider::State,
    f2e_s: nih_widgets::param_slider::State,
    f2e_r: nih_widgets::param_slider::State,

    pe_a: nih_widgets::param_slider::State,
    pe_d: nih_widgets::param_slider::State,
    pe_s: nih_widgets::param_slider::State,
    pe_r: nih_widgets::param_slider::State,
    pe_amt: nih_widgets::param_slider::State,

    master_gain: nih_widgets::param_slider::State,
    num_voices: nih_widgets::param_slider::State,

    preset_pick: pick_list::State<String>,
    selected_preset: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    ParamUpdate(nih_widgets::ParamMessage),
    SelectPreset(String),
}

impl IcedEditor for SynthEditor {
    type Executor = executor::Default;
    type Message = Message;
    type InitializationFlags = Arc<SynthParams>;

    fn new(
        params: Self::InitializationFlags,
        context: Arc<dyn GuiContext>,
    ) -> (Self, Command<Self::Message>) {
        let editor = SynthEditor {
            params,
            context,
            osc1_wave: Default::default(),
            osc1_level: Default::default(),
            osc1_detune: Default::default(),
            osc1_octave: Default::default(),
            osc1_on: Default::default(),
            osc1_unison: Default::default(),
            osc1_spread: Default::default(),
            osc2_wave: Default::default(),
            osc2_level: Default::default(),
            osc2_detune: Default::default(),
            osc2_octave: Default::default(),
            osc2_on: Default::default(),
            osc2_unison: Default::default(),
            osc2_spread: Default::default(),
            flt1_type: Default::default(),
            flt1_cutoff: Default::default(),
            flt1_res: Default::default(),
            flt1_drive: Default::default(),
            flt1_envamt: Default::default(),
            flt1_on: Default::default(),
            flt2_type: Default::default(),
            flt2_cutoff: Default::default(),
            flt2_res: Default::default(),
            flt2_drive: Default::default(),
            flt2_envamt: Default::default(),
            flt2_on: Default::default(),
            amp_a: Default::default(),
            amp_d: Default::default(),
            amp_s: Default::default(),
            amp_r: Default::default(),
            f1e_a: Default::default(),
            f1e_d: Default::default(),
            f1e_s: Default::default(),
            f1e_r: Default::default(),
            f2e_a: Default::default(),
            f2e_d: Default::default(),
            f2e_s: Default::default(),
            f2e_r: Default::default(),
            pe_a: Default::default(),
            pe_d: Default::default(),
            pe_s: Default::default(),
            pe_r: Default::default(),
            pe_amt: Default::default(),
            master_gain: Default::default(),
            num_voices: Default::default(),
            preset_pick: Default::default(),
            selected_preset: None,
        };
        (editor, Command::none())
    }

    fn context(&self) -> &dyn GuiContext {
        self.context.as_ref()
    }

    fn update(
        &mut self,
        _window: &mut WindowQueue,
        message: Self::Message,
    ) -> Command<Self::Message> {
        match message {
            Message::ParamUpdate(m) => self.handle_param_message(m),
            Message::SelectPreset(name) => {
                if let Some(preset) = FACTORY_PRESETS.iter().find(|p| p.name == name) {
                    apply_preset(preset, &self.params, self.context.as_ref());
                }
                self.selected_preset = Some(name);
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let preset_names: Vec<String> =
            FACTORY_PRESETS.iter().map(|p| p.name.to_string()).collect();

        let header = Row::new()
            .align_items(Alignment::Center)
            .spacing(px(16.0))
            .push(
                Text::new("RUHRVIBE")
                    .size(px(26.0))
                    .color(COLOR_HEADER)
                    .width(Length::Fill),
            )
            .push(
                Text::new("Preset")
                    .size(px(16.0))
                    .color(COLOR_TEXT_DIM),
            )
            .push(
                PickList::new(
                    &mut self.preset_pick,
                    preset_names,
                    self.selected_preset.clone(),
                    Message::SelectPreset,
                )
                .text_size(px(15.0))
                .width(Length::Units(px(200.0))),
            );

        let osc1 = section(
            "OSCILLATOR 1",
            COLOR_OSC1,
            Column::new()
                .spacing(px(6.0))
                .push(labeled("Wave", nih_widgets::ParamSlider::new(
                    &mut self.osc1_wave,
                    &self.params.osc1.waveform,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Level", nih_widgets::ParamSlider::new(
                    &mut self.osc1_level,
                    &self.params.osc1.level,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Detune", nih_widgets::ParamSlider::new(
                    &mut self.osc1_detune,
                    &self.params.osc1.detune,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Octave", nih_widgets::ParamSlider::new(
                    &mut self.osc1_octave,
                    &self.params.osc1.octave,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("On", nih_widgets::ParamSlider::new(
                    &mut self.osc1_on,
                    &self.params.osc1.enabled,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Unison", nih_widgets::ParamSlider::new(
                    &mut self.osc1_unison,
                    &self.params.osc1.unison_voices,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Spread", nih_widgets::ParamSlider::new(
                    &mut self.osc1_spread,
                    &self.params.osc1.unison_spread,
                ).text_size(px(14.0)).map(Message::ParamUpdate))),
        );

        let osc2 = section(
            "OSCILLATOR 2",
            COLOR_OSC2,
            Column::new()
                .spacing(px(6.0))
                .push(labeled("Wave", nih_widgets::ParamSlider::new(
                    &mut self.osc2_wave,
                    &self.params.osc2.waveform,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Level", nih_widgets::ParamSlider::new(
                    &mut self.osc2_level,
                    &self.params.osc2.level,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Detune", nih_widgets::ParamSlider::new(
                    &mut self.osc2_detune,
                    &self.params.osc2.detune,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Octave", nih_widgets::ParamSlider::new(
                    &mut self.osc2_octave,
                    &self.params.osc2.octave,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("On", nih_widgets::ParamSlider::new(
                    &mut self.osc2_on,
                    &self.params.osc2.enabled,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Unison", nih_widgets::ParamSlider::new(
                    &mut self.osc2_unison,
                    &self.params.osc2.unison_voices,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Spread", nih_widgets::ParamSlider::new(
                    &mut self.osc2_spread,
                    &self.params.osc2.unison_spread,
                ).text_size(px(14.0)).map(Message::ParamUpdate))),
        );

        let flt1 = section(
            "FILTER 1",
            COLOR_FLT1,
            Column::new()
                .spacing(px(6.0))
                .push(labeled("Type", nih_widgets::ParamSlider::new(
                    &mut self.flt1_type,
                    &self.params.filter1.filter_type,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Cutoff", nih_widgets::ParamSlider::new(
                    &mut self.flt1_cutoff,
                    &self.params.filter1.cutoff,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Res", nih_widgets::ParamSlider::new(
                    &mut self.flt1_res,
                    &self.params.filter1.resonance,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Drive", nih_widgets::ParamSlider::new(
                    &mut self.flt1_drive,
                    &self.params.filter1.drive,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("EnvAmt", nih_widgets::ParamSlider::new(
                    &mut self.flt1_envamt,
                    &self.params.filter1.env_amount,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("On", nih_widgets::ParamSlider::new(
                    &mut self.flt1_on,
                    &self.params.filter1.enabled,
                ).text_size(px(14.0)).map(Message::ParamUpdate))),
        );

        let flt2 = section(
            "FILTER 2",
            COLOR_FLT2,
            Column::new()
                .spacing(px(6.0))
                .push(labeled("Type", nih_widgets::ParamSlider::new(
                    &mut self.flt2_type,
                    &self.params.filter2.filter_type,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Cutoff", nih_widgets::ParamSlider::new(
                    &mut self.flt2_cutoff,
                    &self.params.filter2.cutoff,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Res", nih_widgets::ParamSlider::new(
                    &mut self.flt2_res,
                    &self.params.filter2.resonance,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Drive", nih_widgets::ParamSlider::new(
                    &mut self.flt2_drive,
                    &self.params.filter2.drive,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("EnvAmt", nih_widgets::ParamSlider::new(
                    &mut self.flt2_envamt,
                    &self.params.filter2.env_amount,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("On", nih_widgets::ParamSlider::new(
                    &mut self.flt2_on,
                    &self.params.filter2.enabled,
                ).text_size(px(14.0)).map(Message::ParamUpdate))),
        );

        let amp_env = section(
            "AMP ENVELOPE",
            COLOR_AMP_ENV,
            Column::new()
                .spacing(px(6.0))
                .push(labeled("A", nih_widgets::ParamSlider::new(
                    &mut self.amp_a,
                    &self.params.amp_env.attack,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("D", nih_widgets::ParamSlider::new(
                    &mut self.amp_d,
                    &self.params.amp_env.decay,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("S", nih_widgets::ParamSlider::new(
                    &mut self.amp_s,
                    &self.params.amp_env.sustain,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("R", nih_widgets::ParamSlider::new(
                    &mut self.amp_r,
                    &self.params.amp_env.release,
                ).text_size(px(14.0)).map(Message::ParamUpdate))),
        );

        let f1_env = section(
            "FILTER 1 ENV",
            COLOR_F1_ENV,
            Column::new()
                .spacing(px(6.0))
                .push(labeled("A", nih_widgets::ParamSlider::new(
                    &mut self.f1e_a,
                    &self.params.filter1_env.attack,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("D", nih_widgets::ParamSlider::new(
                    &mut self.f1e_d,
                    &self.params.filter1_env.decay,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("S", nih_widgets::ParamSlider::new(
                    &mut self.f1e_s,
                    &self.params.filter1_env.sustain,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("R", nih_widgets::ParamSlider::new(
                    &mut self.f1e_r,
                    &self.params.filter1_env.release,
                ).text_size(px(14.0)).map(Message::ParamUpdate))),
        );

        let f2_env = section(
            "FILTER 2 ENV",
            COLOR_F2_ENV,
            Column::new()
                .spacing(px(6.0))
                .push(labeled("A", nih_widgets::ParamSlider::new(
                    &mut self.f2e_a,
                    &self.params.filter2_env.attack,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("D", nih_widgets::ParamSlider::new(
                    &mut self.f2e_d,
                    &self.params.filter2_env.decay,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("S", nih_widgets::ParamSlider::new(
                    &mut self.f2e_s,
                    &self.params.filter2_env.sustain,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("R", nih_widgets::ParamSlider::new(
                    &mut self.f2e_r,
                    &self.params.filter2_env.release,
                ).text_size(px(14.0)).map(Message::ParamUpdate))),
        );

        let pitch_env = section(
            "PITCH ENVELOPE",
            COLOR_PITCH_ENV,
            Column::new()
                .spacing(px(6.0))
                .push(labeled("A", nih_widgets::ParamSlider::new(
                    &mut self.pe_a,
                    &self.params.pitch_env.attack,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("D", nih_widgets::ParamSlider::new(
                    &mut self.pe_d,
                    &self.params.pitch_env.decay,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("S", nih_widgets::ParamSlider::new(
                    &mut self.pe_s,
                    &self.params.pitch_env.sustain,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("R", nih_widgets::ParamSlider::new(
                    &mut self.pe_r,
                    &self.params.pitch_env.release,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Amount", nih_widgets::ParamSlider::new(
                    &mut self.pe_amt,
                    &self.params.pitch_env.amount,
                ).text_size(px(14.0)).map(Message::ParamUpdate))),
        );

        let master = section(
            "MASTER",
            COLOR_MASTER,
            Column::new()
                .spacing(px(6.0))
                .push(labeled("Gain", nih_widgets::ParamSlider::new(
                    &mut self.master_gain,
                    &self.params.master_gain,
                ).text_size(px(14.0)).map(Message::ParamUpdate)))
                .push(labeled("Voices", nih_widgets::ParamSlider::new(
                    &mut self.num_voices,
                    &self.params.num_voices,
                ).text_size(px(14.0)).map(Message::ParamUpdate))),
        );

        let row_oscs = Row::new().spacing(px(12.0)).push(osc1).push(osc2);
        let row_filters = Row::new().spacing(px(12.0)).push(flt1).push(flt2);
        let row_envs = Row::new()
            .spacing(px(12.0))
            .push(amp_env)
            .push(f1_env)
            .push(f2_env)
            .push(pitch_env);
        let row_master = Row::new().spacing(px(12.0)).push(master);

        let content = Column::new()
            .padding(px(14.0))
            .spacing(px(12.0))
            .push(header)
            .push(row_oscs)
            .push(row_filters)
            .push(row_envs)
            .push(row_master);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(px(8.0))
            .style(PanelStyle)
            .into()
    }

    fn background_color(&self) -> Color {
        COLOR_BG
    }
}

impl SynthEditor {
    fn handle_param_message(&self, message: nih_widgets::ParamMessage) {
        match message {
            nih_widgets::ParamMessage::BeginSetParameter(p) => unsafe {
                self.context.raw_begin_set_parameter(p);
            },
            nih_widgets::ParamMessage::SetParameterNormalized(p, v) => unsafe {
                self.context.raw_set_parameter_normalized(p, v);
            },
            nih_widgets::ParamMessage::EndSetParameter(p) => unsafe {
                self.context.raw_end_set_parameter(p);
            },
        }
    }
}

fn labeled<'a>(label: &str, widget: Element<'a, Message>) -> Row<'a, Message> {
    Row::new()
        .align_items(Alignment::Center)
        .spacing(px(8.0))
        .push(
            Text::new(label.to_string())
                .size(px(14.0))
                .color(COLOR_TEXT)
                .width(Length::Units(px(64.0))),
        )
        .push(widget)
}

fn section<'a>(title: &str, accent: Color, content: Column<'a, Message>) -> Element<'a, Message> {
    let title_text = Text::new(title.to_string())
        .size(px(15.0))
        .color(accent);

    let inner = Column::new()
        .spacing(px(8.0))
        .push(title_text)
        .push(content);

    Container::new(inner)
        .padding(px(12.0))
        .width(Length::Fill)
        .style(SectionStyle { accent })
        .into()
}
