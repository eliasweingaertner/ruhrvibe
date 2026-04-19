//! Vizia-based GUI editor for the subtractive synth.
//!
//! Vizia handles HiDPI natively: sizes are in logical pixels and the
//! framework scales based on host/system DPI.

use nih_plug::prelude::{Editor, GuiContext, Param};
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;

use crate::params::SynthParams;
use crate::presets::{apply_preset, presets_in_category, CATEGORIES, FACTORY_PRESETS};

const WAVEFORM_LABELS: &[&str] = &["Sine", "Saw", "Squ", "Tri", "Noise"];
const FILTER_TYPE_LABELS: &[&str] = &["LP", "HP", "BP", "Notch"];

const BASE_WIDTH: u32 = 820;
const BASE_HEIGHT: u32 = 980;

// Direct overrides of nih_plug_vizia's built-in widget stylesheet. These are
// simple-type selectors so they override the defaults with equal specificity
// (our stylesheet is added after widgets::register_theme).
const STYLE: &str = r#"
/* Global defaults so the custom theme background applies everywhere */
:root {
    background-color: #23262E;
    color: #E8EAF0;
    font-size: 12;
}

/* Shrink nih_plug_vizia defaults so they fit our compact layout */
param-slider {
    width: 1s;
    height: 22px;
    border-color: #1A1C22;
    border-width: 1px;
    background-color: #1E2028;
}
param-slider:hover,
param-slider:active {
    background-color: #2A2D36;
}
param-slider .fill {
    background-color: #F0A526;
}
param-slider .value-entry {
    color: #E8EAF0;
    font-size: 12;
    background-color: transparent;
    border-width: 0px;
    child-space: 1s;
}

/* Root panel */
.root {
    background-color: #23262E;
    child-space: 8px;
    row-between: 6px;
    width: 1s;
    height: 1s;
}

/* Header row */
.header {
    height: 28px;
    col-between: 6px;
    child-top: 1s;
    child-bottom: 1s;
}
.header-title {
    color: #F0A526;
    font-size: 18;
    width: auto;
    height: auto;
    child-top: 1s;
    child-bottom: 1s;
}

/* Section box */
.section {
    background-color: #2C3038;
    border-radius: 6px;
    border-width: 1.5px;
    border-color: #3A4050;
    child-space: 6px;
    row-between: 2px;
    height: auto;
    width: 1s;
}
.section-title {
    font-size: 13;
    width: 1s;
    height: 16px;
    child-bottom: 2px;
}

/* Accent colors for the section title + border */
.accent-osc1 { border-color: #D8961F; }
.accent-osc1 .section-title { color: #F0A526; }
.accent-osc2 { border-color: #B57918; }
.accent-osc2 .section-title { color: #D8901B; }
.accent-flt1 { border-color: #40648C; }
.accent-flt1 .section-title { color: #5887B0; }
.accent-flt2 { border-color: #587F9F; }
.accent-flt2 .section-title { color: #7298B8; }
.accent-amp  { border-color: #D0842B; }
.accent-amp  .section-title { color: #F09428; }
.accent-fe1  { border-color: #4D7293; }
.accent-fe1  .section-title { color: #6890B0; }
.accent-fe2  { border-color: #A27E1E; }
.accent-fe2  .section-title { color: #C09528; }
.accent-pe   { border-color: #7993AE; }
.accent-pe   .section-title { color: #95B0C8; }
.accent-mst  { border-color: #B08823; }
.accent-mst  .section-title { color: #D0A028; }
.accent-chr  { border-color: #6B8E6F; }
.accent-chr  .section-title { color: #8FBF95; }
.accent-dly  { border-color: #7D6BA0; }
.accent-dly  .section-title { color: #A78FC8; }

/* A labeled parameter row: [Label | ParamSlider] */
.param-row {
    height: 24px;
    width: 1s;
    col-between: 6px;
    child-top: 1s;
    child-bottom: 1s;
}
.param-label {
    width: 56px;
    height: 1s;
    color: #E8EAF0;
    font-size: 11;
    child-top: 1s;
    child-bottom: 1s;
}

/* Preset pickers & buttons */
.cat-pick {
    width: 100px;
    height: 26px;
}
.preset-name-pick {
    width: 240px;
    height: 26px;
}
picklist {
    height: 26px;
    background-color: #1E2028;
    border-width: 1px;
    border-color: #1A1C22;
    border-radius: 3px;
    color: #E8EAF0;
    font-size: 12;
}
.nav-btn {
    width: 26px;
    height: 26px;
    child-space: 1s;
    background-color: #2C3038;
    border-color: #3A4050;
    border-width: 1px;
    border-radius: 4px;
    color: #E8EAF0;
}
.nav-btn:hover {
    background-color: #3A4050;
}

/* Row wrappers */
.row-equal {
    col-between: 6px;
    height: auto;
    width: 1s;
}

/* High-contrast dropdown for preset pickers (vizia defaults are light-on-light) */
dropdown {
    background-color: #1E2028;
    border-radius: 4px;
    border-width: 1px;
    border-color: #3A4050;
    color: #E8EAF0;
}
dropdown:hover {
    background-color: #2A2D36;
}
dropdown .title {
    color: #E8EAF0;
    background-color: transparent;
    border-width: 0px;
    child-space: 4px;
}
dropdown popup {
    background-color: #23262E;
    border-radius: 4px;
    border-width: 1px;
    border-color: #3A4050;
    outer-shadow: 0 3 10 #00000080;
}
dropdown list label {
    color: #E8EAF0;
    background-color: #23262E;
    height: 24px;
    font-size: 12;
    child-left: 8px;
    child-right: 8px;
    child-top: 1s;
    child-bottom: 1s;
}
dropdown list label:hover {
    background-color: #3A4050;
    color: #FFFFFF;
}
dropdown list label:checked {
    background-color: #F0A526;
    color: #1E2028;
}
picklist label,
picklist .icon {
    color: #E8EAF0;
}

/* Radio group for enum parameters */
radio-group {
    layout-type: row;
    height: 22px;
    width: 1s;
    col-between: 2px;
}
radio-group label {
    width: 1s;
    height: 1s;
    child-space: 1s;
    font-size: 11;
    color: #B8BAC0;
    background-color: #1E2028;
    border-width: 1px;
    border-color: #1A1C22;
    border-radius: 3px;
}
radio-group label:hover {
    background-color: #2A2D36;
    color: #FFFFFF;
}
radio-group label:checked {
    background-color: #F0A526;
    color: #1E2028;
    border-color: #B88018;
}
radio-group label:checked:hover {
    background-color: #FFB833;
}
"#;

#[derive(Lens)]
struct AppData {
    params: Arc<SynthParams>,
    gui_context: Arc<dyn GuiContext>,

    categories: Vec<String>,
    selected_category_idx: usize,

    preset_names: Vec<String>,
    selected_preset_idx: usize,
}

#[derive(Debug)]
enum AppEvent {
    SelectCategory(usize),
    SelectPreset(usize),
    PrevPreset,
    NextPreset,
}

impl AppData {
    fn refresh_preset_list(&mut self) {
        let cat = &self.categories[self.selected_category_idx];
        self.preset_names = presets_in_category(cat)
            .iter()
            .map(|p| p.name.to_string())
            .collect();
    }

    fn apply_selected(&self) {
        let cat = &self.categories[self.selected_category_idx];
        let presets = presets_in_category(cat);
        if let Some(p) = presets.get(self.selected_preset_idx) {
            apply_preset(p, &self.params, self.gui_context.as_ref());
        }
    }
}

impl Model for AppData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::SelectCategory(idx) => {
                self.selected_category_idx = *idx;
                self.selected_preset_idx = 0;
                self.refresh_preset_list();
                // Don't auto-apply when category changes — user may be browsing.
            }
            AppEvent::SelectPreset(idx) => {
                self.selected_preset_idx = *idx;
                self.apply_selected();
            }
            AppEvent::PrevPreset => {
                let len = self.preset_names.len();
                if len > 0 {
                    self.selected_preset_idx = if self.selected_preset_idx == 0 {
                        len - 1
                    } else {
                        self.selected_preset_idx - 1
                    };
                    self.apply_selected();
                }
            }
            AppEvent::NextPreset => {
                let len = self.preset_names.len();
                if len > 0 {
                    self.selected_preset_idx = (self.selected_preset_idx + 1) % len;
                    self.apply_selected();
                }
            }
        });
    }
}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (BASE_WIDTH, BASE_HEIGHT))
}

pub(crate) fn create(
    params: Arc<SynthParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, gui_context| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);
        cx.add_stylesheet(STYLE).expect("invalid stylesheet");

        let categories: Vec<String> = CATEGORIES.iter().map(|c| c.to_string()).collect();
        let preset_names: Vec<String> = presets_in_category(CATEGORIES[0])
            .iter()
            .map(|p| p.name.to_string())
            .collect();

        AppData {
            params: params.clone(),
            gui_context,
            categories,
            selected_category_idx: 0,
            preset_names,
            selected_preset_idx: 0,
        }
        .build(cx);

        VStack::new(cx, |cx| {
            build_header(cx);
            build_osc_row(cx);
            build_filter_row(cx);
            build_env_row(cx);
            build_fx_row(cx);
            build_bottom_row(cx);
        })
        .class("root");

        ResizeHandle::new(cx);
    })
}

fn build_header(cx: &mut Context) {
    HStack::new(cx, |cx| {
        Label::new(cx, "RUHRVIBE").class("header-title");

        // Spacer
        Element::new(cx).width(Stretch(1.0)).height(Pixels(1.0));

        // Category picker
        PickList::new(
            cx,
            AppData::categories,
            AppData::selected_category_idx,
            true,
        )
        .on_select(|cx, idx| cx.emit(AppEvent::SelectCategory(idx)))
        .class("cat-pick");

        // Prev button
        Button::new(
            cx,
            |cx| cx.emit(AppEvent::PrevPreset),
            |cx| Label::new(cx, "<"),
        )
        .class("nav-btn");

        // Preset picker
        PickList::new(
            cx,
            AppData::preset_names,
            AppData::selected_preset_idx,
            true,
        )
        .on_select(|cx, idx| cx.emit(AppEvent::SelectPreset(idx)))
        .class("preset-name-pick");

        // Next button
        Button::new(
            cx,
            |cx| cx.emit(AppEvent::NextPreset),
            |cx| Label::new(cx, ">"),
        )
        .class("nav-btn");
    })
    .class("header");
}

fn build_osc_row(cx: &mut Context) {
    HStack::new(cx, |cx| {
        osc_section(cx, "OSCILLATOR 1", "accent-osc1", OscSel::Osc1);
        osc_section(cx, "OSCILLATOR 2", "accent-osc2", OscSel::Osc2);
    })
    .class("row-equal");
}

fn build_filter_row(cx: &mut Context) {
    HStack::new(cx, |cx| {
        filter_section(cx, "FILTER 1", "accent-flt1", FilterSel::Flt1);
        filter_section(cx, "FILTER 2", "accent-flt2", FilterSel::Flt2);
    })
    .class("row-equal");
}

fn build_env_row(cx: &mut Context) {
    HStack::new(cx, |cx| {
        env_section(cx, "AMP ENVELOPE", "accent-amp", EnvSel::Amp);
        env_section(cx, "FILTER 1 ENV", "accent-fe1", EnvSel::F1);
        env_section(cx, "FILTER 2 ENV", "accent-fe2", EnvSel::F2);
    })
    .class("row-equal");
}

fn build_fx_row(cx: &mut Context) {
    HStack::new(cx, |cx| {
        chorus_section(cx);
        delay_section(cx);
    })
    .class("row-equal");
}

fn build_bottom_row(cx: &mut Context) {
    HStack::new(cx, |cx| {
        pitch_env_section(cx);
        master_section(cx);
    })
    .class("row-equal");
}

#[derive(Clone, Copy)]
enum OscSel { Osc1, Osc2 }

#[derive(Clone, Copy)]
enum FilterSel { Flt1, Flt2 }

#[derive(Clone, Copy)]
enum EnvSel { Amp, F1, F2 }

fn section_container<F>(cx: &mut Context, title: &str, accent_class: &'static str, content: F)
where
    F: FnOnce(&mut Context),
{
    VStack::new(cx, |cx| {
        Label::new(cx, title).class("section-title");
        content(cx);
    })
    .class("section")
    .class(accent_class);
}

fn labeled_row<F>(cx: &mut Context, label: &str, content: F)
where
    F: FnOnce(&mut Context),
{
    HStack::new(cx, |cx| {
        Label::new(cx, label).class("param-label");
        content(cx);
    })
    .class("param-row");
}

fn osc_section(cx: &mut Context, title: &str, accent: &'static str, sel: OscSel) {
    section_container(cx, title, accent, move |cx| {
        labeled_row(cx, "Wave", move |cx| {
            match sel {
                OscSel::Osc1 => { RadioGroup::new(cx, AppData::params, |p| &p.osc1.waveform, WAVEFORM_LABELS); }
                OscSel::Osc2 => { RadioGroup::new(cx, AppData::params, |p| &p.osc2.waveform, WAVEFORM_LABELS); }
            }
        });
        labeled_row(cx, "Level", move |cx| {
            match sel {
                OscSel::Osc1 => { ParamSlider::new(cx, AppData::params, |p| &p.osc1.level); }
                OscSel::Osc2 => { ParamSlider::new(cx, AppData::params, |p| &p.osc2.level); }
            }
        });
        labeled_row(cx, "Detune", move |cx| {
            match sel {
                OscSel::Osc1 => { ParamSlider::new(cx, AppData::params, |p| &p.osc1.detune); }
                OscSel::Osc2 => { ParamSlider::new(cx, AppData::params, |p| &p.osc2.detune); }
            }
        });
        labeled_row(cx, "Octave", move |cx| {
            match sel {
                OscSel::Osc1 => { ParamSlider::new(cx, AppData::params, |p| &p.osc1.octave); }
                OscSel::Osc2 => { ParamSlider::new(cx, AppData::params, |p| &p.osc2.octave); }
            }
        });
        labeled_row(cx, "On", move |cx| {
            match sel {
                OscSel::Osc1 => { ParamSlider::new(cx, AppData::params, |p| &p.osc1.enabled); }
                OscSel::Osc2 => { ParamSlider::new(cx, AppData::params, |p| &p.osc2.enabled); }
            }
        });
        labeled_row(cx, "Unison", move |cx| {
            match sel {
                OscSel::Osc1 => { ParamSlider::new(cx, AppData::params, |p| &p.osc1.unison_voices); }
                OscSel::Osc2 => { ParamSlider::new(cx, AppData::params, |p| &p.osc2.unison_voices); }
            }
        });
        labeled_row(cx, "Spread", move |cx| {
            match sel {
                OscSel::Osc1 => { ParamSlider::new(cx, AppData::params, |p| &p.osc1.unison_spread); }
                OscSel::Osc2 => { ParamSlider::new(cx, AppData::params, |p| &p.osc2.unison_spread); }
            }
        });
        labeled_row(cx, "Pan", move |cx| {
            match sel {
                OscSel::Osc1 => { ParamSlider::new(cx, AppData::params, |p| &p.osc1.pan); }
                OscSel::Osc2 => { ParamSlider::new(cx, AppData::params, |p| &p.osc2.pan); }
            }
        });
        labeled_row(cx, "Stereo", move |cx| {
            match sel {
                OscSel::Osc1 => { ParamSlider::new(cx, AppData::params, |p| &p.osc1.stereo_spread); }
                OscSel::Osc2 => { ParamSlider::new(cx, AppData::params, |p| &p.osc2.stereo_spread); }
            }
        });
    });
}

fn filter_section(cx: &mut Context, title: &str, accent: &'static str, sel: FilterSel) {
    section_container(cx, title, accent, move |cx| {
        labeled_row(cx, "Type", move |cx| {
            match sel {
                FilterSel::Flt1 => { RadioGroup::new(cx, AppData::params, |p| &p.filter1.filter_type, FILTER_TYPE_LABELS); }
                FilterSel::Flt2 => { RadioGroup::new(cx, AppData::params, |p| &p.filter2.filter_type, FILTER_TYPE_LABELS); }
            }
        });
        labeled_row(cx, "Cutoff", move |cx| {
            match sel {
                FilterSel::Flt1 => { ParamSlider::new(cx, AppData::params, |p| &p.filter1.cutoff); }
                FilterSel::Flt2 => { ParamSlider::new(cx, AppData::params, |p| &p.filter2.cutoff); }
            }
        });
        labeled_row(cx, "Res", move |cx| {
            match sel {
                FilterSel::Flt1 => { ParamSlider::new(cx, AppData::params, |p| &p.filter1.resonance); }
                FilterSel::Flt2 => { ParamSlider::new(cx, AppData::params, |p| &p.filter2.resonance); }
            }
        });
        labeled_row(cx, "Drive", move |cx| {
            match sel {
                FilterSel::Flt1 => { ParamSlider::new(cx, AppData::params, |p| &p.filter1.drive); }
                FilterSel::Flt2 => { ParamSlider::new(cx, AppData::params, |p| &p.filter2.drive); }
            }
        });
        labeled_row(cx, "EnvAmt", move |cx| {
            match sel {
                FilterSel::Flt1 => { ParamSlider::new(cx, AppData::params, |p| &p.filter1.env_amount); }
                FilterSel::Flt2 => { ParamSlider::new(cx, AppData::params, |p| &p.filter2.env_amount); }
            }
        });
        labeled_row(cx, "On", move |cx| {
            match sel {
                FilterSel::Flt1 => { ParamSlider::new(cx, AppData::params, |p| &p.filter1.enabled); }
                FilterSel::Flt2 => { ParamSlider::new(cx, AppData::params, |p| &p.filter2.enabled); }
            }
        });
    });
}

fn env_section(cx: &mut Context, title: &str, accent: &'static str, sel: EnvSel) {
    section_container(cx, title, accent, move |cx| {
        labeled_row(cx, "A", move |cx| {
            match sel {
                EnvSel::Amp => { ParamSlider::new(cx, AppData::params, |p| &p.amp_env.attack); }
                EnvSel::F1  => { ParamSlider::new(cx, AppData::params, |p| &p.filter1_env.attack); }
                EnvSel::F2  => { ParamSlider::new(cx, AppData::params, |p| &p.filter2_env.attack); }
            }
        });
        labeled_row(cx, "D", move |cx| {
            match sel {
                EnvSel::Amp => { ParamSlider::new(cx, AppData::params, |p| &p.amp_env.decay); }
                EnvSel::F1  => { ParamSlider::new(cx, AppData::params, |p| &p.filter1_env.decay); }
                EnvSel::F2  => { ParamSlider::new(cx, AppData::params, |p| &p.filter2_env.decay); }
            }
        });
        labeled_row(cx, "S", move |cx| {
            match sel {
                EnvSel::Amp => { ParamSlider::new(cx, AppData::params, |p| &p.amp_env.sustain); }
                EnvSel::F1  => { ParamSlider::new(cx, AppData::params, |p| &p.filter1_env.sustain); }
                EnvSel::F2  => { ParamSlider::new(cx, AppData::params, |p| &p.filter2_env.sustain); }
            }
        });
        labeled_row(cx, "R", move |cx| {
            match sel {
                EnvSel::Amp => { ParamSlider::new(cx, AppData::params, |p| &p.amp_env.release); }
                EnvSel::F1  => { ParamSlider::new(cx, AppData::params, |p| &p.filter1_env.release); }
                EnvSel::F2  => { ParamSlider::new(cx, AppData::params, |p| &p.filter2_env.release); }
            }
        });
    });
}

fn pitch_env_section(cx: &mut Context) {
    section_container(cx, "PITCH ENVELOPE", "accent-pe", |cx| {
        labeled_row(cx, "A",      |cx| { ParamSlider::new(cx, AppData::params, |p| &p.pitch_env.attack); });
        labeled_row(cx, "D",      |cx| { ParamSlider::new(cx, AppData::params, |p| &p.pitch_env.decay); });
        labeled_row(cx, "S",      |cx| { ParamSlider::new(cx, AppData::params, |p| &p.pitch_env.sustain); });
        labeled_row(cx, "R",      |cx| { ParamSlider::new(cx, AppData::params, |p| &p.pitch_env.release); });
        labeled_row(cx, "Amount", |cx| { ParamSlider::new(cx, AppData::params, |p| &p.pitch_env.amount); });
    });
}

fn master_section(cx: &mut Context) {
    section_container(cx, "MASTER", "accent-mst", |cx| {
        labeled_row(cx, "Gain",   |cx| { ParamSlider::new(cx, AppData::params, |p| &p.master_gain); });
        labeled_row(cx, "Voices", |cx| { ParamSlider::new(cx, AppData::params, |p| &p.num_voices); });
    });
}

fn chorus_section(cx: &mut Context) {
    section_container(cx, "CHORUS", "accent-chr", |cx| {
        labeled_row(cx, "On",    |cx| { ParamSlider::new(cx, AppData::params, |p| &p.chorus.enabled); });
        labeled_row(cx, "Rate",  |cx| { ParamSlider::new(cx, AppData::params, |p| &p.chorus.rate); });
        labeled_row(cx, "Depth", |cx| { ParamSlider::new(cx, AppData::params, |p| &p.chorus.depth); });
        labeled_row(cx, "Mix",   |cx| { ParamSlider::new(cx, AppData::params, |p| &p.chorus.mix); });
    });
}

fn delay_section(cx: &mut Context) {
    section_container(cx, "DELAY (PING-PONG)", "accent-dly", |cx| {
        labeled_row(cx, "On",       |cx| { ParamSlider::new(cx, AppData::params, |p| &p.delay.enabled); });
        labeled_row(cx, "Time",     |cx| { ParamSlider::new(cx, AppData::params, |p| &p.delay.time_ms); });
        labeled_row(cx, "Feedback", |cx| { ParamSlider::new(cx, AppData::params, |p| &p.delay.feedback); });
        labeled_row(cx, "Tone",     |cx| { ParamSlider::new(cx, AppData::params, |p| &p.delay.tone); });
        labeled_row(cx, "Mix",      |cx| { ParamSlider::new(cx, AppData::params, |p| &p.delay.mix); });
    });
}

// Silence warnings about unused constant for 157-preset count.
#[allow(dead_code)]
const _TOTAL_PRESETS: usize = FACTORY_PRESETS.len();

// --- Radio group widget for discrete enum parameters ---

#[derive(Debug)]
enum RadioEvent {
    Set(f32),
}

pub struct RadioGroup {
    param_base: ParamWidgetBase,
}

impl RadioGroup {
    pub fn new<'a, L, P, FMap>(
        cx: &'a mut Context,
        params: L,
        params_to_param: FMap,
        labels: &'static [&'static str],
    ) -> Handle<'a, Self>
    where
        L: Lens<Target = Arc<SynthParams>> + Copy,
        P: Param + 'static,
        FMap: Fn(&Arc<SynthParams>) -> &P + Copy + 'static,
    {
        Self {
            param_base: ParamWidgetBase::new(cx, params, params_to_param),
        }
        .build(
            cx,
            ParamWidgetBase::build_view(params, params_to_param, move |cx, param_data| {
                let count = labels.len().max(1);
                let step_denom = (count.saturating_sub(1)).max(1) as f32;
                let current = param_data.make_lens(|p| p.unmodulated_normalized_value());
                let epsilon = 0.5 / step_denom;
                for (i, &label) in labels.iter().enumerate() {
                    let value = i as f32 / step_denom;
                    Label::new(cx, label)
                        .checked(current.map(move |v| (*v - value).abs() < epsilon))
                        .on_press(move |cx| {
                            cx.emit(RadioEvent::Set(value));
                        });
                }
            }),
        )
    }
}

impl View for RadioGroup {
    fn element(&self) -> Option<&'static str> {
        Some("radio-group")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e: &RadioEvent, meta| match e {
            RadioEvent::Set(v) => {
                self.param_base.begin_set_parameter(cx);
                self.param_base.set_normalized_value(cx, *v);
                self.param_base.end_set_parameter(cx);
                meta.consume();
            }
        });
    }
}
