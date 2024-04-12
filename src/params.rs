#![allow(dead_code)]

use fieldset::*;

use crate::bounded::{Bounded, Norm, SNorm};

#[derive(Debug, Clone, Copy)]
pub enum Channel {
    Omni,
    Ch1,
    Ch2,
    Ch3,
    Ch4,
    Ch5,
    Ch6,
    Ch7,
    Ch8,
    Ch9,
    Ch10,
    Ch11,
    Ch12,
    Ch13,
    Ch14,
    Ch15,
    Ch16,
    Mpe,
}

#[derive(Debug, Clone, Copy)]
pub enum ControlMode {
    Jump,
    Catch,
    Scale,
}

#[derive(Debug, Clone, Copy)]
pub enum DriveMode {
    Low,
    Mid,
    High,
}

#[derive(Debug, Clone, Copy)]
pub enum LfoSync {
    Free,
    Key,
    Bpm,
    BpmKey,
}

#[derive(Debug, Clone, Copy)]
pub enum LfoWave {
    Sine,
    Triangle,
    Saw,
    Ramp,
    Square,
    Random,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayMode {
    Poly,
    Unison,
    Solo,
}

#[derive(Debug, Clone, Copy)]
pub enum TrackMode {
    Off,
    Mid,
    High,
}

#[derive(Debug, Clone, Copy, FieldSet)]
pub struct Coefficients {
    pub vco_tune_1: SNorm,
    pub vco_tune_2: SNorm,
    pub vco_wave_1: SNorm,
    pub vco_wave_2: SNorm,
    pub vco_mix: SNorm,
    pub vco_mod_1: SNorm,
    pub vco_mod_2: SNorm,
    pub vco_detune: SNorm,
    pub vco_glide: SNorm,
    pub filter_cutoff: SNorm,
    pub filter_reson: SNorm,
    pub filter_env: SNorm,
    pub filter_lfo: SNorm,
    pub filter_bass: SNorm,
    pub filter_fm: SNorm,
    pub master_spread: SNorm,
    pub bbd_amount: SNorm,
    pub bbd_morph: SNorm,
    pub delay_mix: SNorm,
    pub delay_time: SNorm,
    pub delay_feed: SNorm,
    pub poly_lfo_fade: SNorm,
    pub poly_lfo_rate: SNorm,
    pub poly_lfo_spread: SNorm,
    pub eg_attack: SNorm,
    pub eg_decay: SNorm,
    pub eg_sustain: SNorm,
    pub eg_release: SNorm,
    pub vca_eg_attack: SNorm,
    pub vca_eg_decay: SNorm,
    pub vca_eg_sustain: SNorm,
    pub vca_eg_release: SNorm,
    pub lfo_b_rate: SNorm,
    pub lfo_b_amount: SNorm,
}

#[derive(Debug, Clone, Copy, FieldSet)]
pub struct BaseParameters {
    pub vco_tune_1: Norm,
    pub vco_tune_2: Norm,
    pub vco_wave_1: Norm,
    pub vco_wave_2: Norm,
    pub vco_mix: Norm,
    pub vco_mod_1: Norm,
    pub vco_mod_2: Norm,
    pub vco_detune: Norm,
    pub vco_glide: Norm,
    pub filter_cutoff: Norm,
    pub filter_reson: Norm,
    pub filter_env: Norm,
    pub filter_lfo: Norm,
    pub filter_bass: Norm,
    pub filter_fm: Norm,
    pub master_spread: Norm,
    pub bbd_amount: Norm,
    pub bbd_morph: Norm,
    pub delay_mix: Norm,
    pub delay_time: Norm,
    pub delay_feed: Norm,
    pub poly_lfo_fade: Norm,
    pub poly_lfo_rate: Norm,
    pub poly_lfo_spread: Norm,
    pub eg_attack: Norm,
    pub eg_decay: Norm,
    pub eg_sustain: Norm,
    pub eg_release: Norm,
    pub vca_eg_attack: Norm,
    pub vca_eg_decay: Norm,
    pub vca_eg_sustain: Norm,
    pub vca_eg_release: Norm,
    pub lfo_b_rate: Norm,
    pub lfo_b_amount: Norm,
    pub lfo_b_wave: LfoWave,
    pub poly_lfo_wave: LfoWave,
    pub sync: bool,
    pub track: TrackMode,
}

#[derive(Debug, Clone, Copy, FieldSet)]
pub struct MenuParameters {
    pub play_mode: PlayMode,
    #[fieldset]
    pub mod_wheel_coefficients: Coefficients,
    #[fieldset]
    pub velocity_coefficients: Coefficients,
    #[fieldset]
    pub aftertouch_coefficients: Coefficients,
    pub drive_mode: DriveMode,
    pub poly_lfo_sync: LfoSync,
    pub global_lfo_sync: LfoSync,
    pub amp_velocity: Norm,
    pub legato: bool,
    pub amp_level: Norm,
}

pub type PitchWheel = Bounded<i8, 0, 12>;
pub type PitchWheelMpe = Bounded<i8, 0, 48>;

#[derive(Debug, Clone, Copy, FieldSet)]
pub struct GlobalSettings {
    pub channel: Channel,
    pub cc_in: bool,
    pub cc_out: bool,
    pub pc_in: bool,
    pub pc_out: bool,
    pub fine_tune: SNorm,
    pub control_mode: ControlMode,
    pub load_preview: bool,
    pub pitch_wheel: PitchWheel,
    pub pitch_wheel_mpe: PitchWheelMpe,
}

#[derive(Debug, Clone, Copy, FieldSet)]
pub struct Parameters {
    #[fieldset]
    pub lfo_coefficients: Coefficients,
    #[fieldset]
    pub base_parameters: BaseParameters,
    #[fieldset]
    pub menu_parameters: MenuParameters,
    #[fieldset]
    pub global_settings: GlobalSettings,
}
