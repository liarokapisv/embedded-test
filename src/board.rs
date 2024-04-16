#![allow(dead_code)]

use core::time::Duration;

use crate::bounded::Bounded;
use crate::bounded::Norm;

const NUMBER_OF_VOICES: usize = 8;

pub trait Button {
    fn get(&self) -> bool;
}

pub trait Codec {
    fn prepare(&mut self, sample_rate: f32, block_size: usize);
    fn process(&mut self, in_samples: &[f32], out_samples: &mut [f32]);
}

pub trait OpaqueControl {
    fn set(&mut self, value: Norm);
}

pub type DacValue = Bounded<u16, 0, 1023>;

pub trait Dac {
    fn set(&mut self, value: DacValue);
}

impl<T: Dac + PartialOrd + Copy> OpaqueControl for T {
    fn set(&mut self, value: Norm) {
        <Self as Dac>::set(self, Bounded::new((value.get() * 1023.) as u16).unwrap());
    }
}

const DISPLAY_ROWS: usize = 64;
const DISPLAY_COLUMNS: usize = 128;
type DisplayBuffer = [[bool; DISPLAY_ROWS]; DISPLAY_COLUMNS];

pub trait Display {
    fn buffer(&self) -> &DisplayBuffer;
    fn buffer_mut(&mut self) -> &mut DisplayBuffer;
    fn flush(&mut self);
}

pub trait ExtMemory {
    type Error;
    async fn write(&mut self, sector_id: u8, data: &[u8; 4096]) -> Result<(), Self::Error>;
    async fn read(&mut self, sector_id: u8, data: &mut [u8; 4096]) -> Result<(), Self::Error>;
}

pub trait Gpio {
    fn set(&mut self, set: bool);
}

pub trait Led: Gpio {
    fn toggle(&mut self);
}

pub trait Slider {
    fn get(&self) -> Norm;
}

pub struct MidiMessage;

pub trait RxMidiStream {
    async fn read(&mut self) -> MidiMessage;
}

pub trait TxMidiStream {
    async fn write(&mut self, message: MidiMessage);
}

pub trait IsrListener {
    fn frequency_changed(&mut self, sample_rate: f32);
    fn process(&mut self);
}

pub trait Isr<'d> {
    fn start(&mut self, callback: &mut (dyn IsrListener + 'd));
}

pub trait OscFeedback {
    async fn measure(&mut self) -> Duration;
}

pub struct Sliders<T: Slider> {
    pub lfo_1_rate: T,
    pub lfo_1_fade: T,
    pub lfo_1_vco_amount: T,
    pub lfo_1_vcf_amount: T,
    pub lfo_2_rate: T,
    pub lfo_2_xmod: T,
    pub lfo_2_morph_amount: T,
    pub lfo_2_vco_1_pw_amount: T,
    pub vco_sub_noise: T,
    pub vco_mix: T,
    pub vco_morph: T,
    pub vco_2_tune: T,
    pub vco_detune: T,
    pub vco_glide: T,
    pub vco_pw: T,
    pub vco_fm: T,
    pub lpf_cut: T,
    pub lpf_cut_eg_amount: T,
    pub lpf_reson: T,
    pub lpf_ffm: T,
    pub hpf_cut: T,
    pub hpf_reson: T,
    pub master_volume: T,
    pub spread: T,
    pub s_a: T,
    pub s_d: T,
    pub s_s: T,
    pub s_r: T,
    pub s_1: T,
    pub s_2: T,
    pub s_3: T,
    pub s_4: T,
    pub s_5: T,
}

pub struct ButtonLike<T> {
    pub lfo_1_wave: T,
    pub lfo_1_vco_target: T,
    pub lfo_2_wave: T,
    pub lfo_2_sync_mode: T,
    pub vco_sync: T,
    pub vco_fm_eg: T,
    pub lpf_poles: T,
    pub lpf_track: T,
    pub lpf_ffm_noise_source: T,
    pub eg_mode: T,

    pub play: T,
    pub stop: T,
    pub rec: T,
    pub menu: T,
    pub save: T,
    pub dist: T,
    pub modw: T,
    pub delay: T,
    pub rev: T,
    pub seq: T,
}

pub type Buttons<T> = ButtonLike<T>;
pub type Leds<T> = ButtonLike<T>;

struct GlobalControls<O: OpaqueControl, G: Gpio> {
    pub vco_1_level: O,
    pub vco_sub_noise: O,
    pub vco_2_pw: O,
    pub vco_2_level: O,
    pub vco_sync: O,
    pub lpf_reson: O,
    pub lpf_ffm_noise_source: G,
    pub lpf_ffm: O,
    pub lpf_poles: G,
}

struct VoiceControls<O: OpaqueControl, D: Dac> {
    pub vco_1_tune_main: D,
    pub vco_1_tune_offset: D,
    pub vco_2_tune_main: D,
    pub vco_2_tune_offset: D,
    pub vco_1_pw: O,
    pub vco_fm: O,
    pub vco_morph: O,
    pub lpf_cut_main: D,
    pub lpf_cut_offset: D,
    pub vca_eg: O,
}

struct Controls<O: OpaqueControl, G: Gpio, D: Dac> {
    pub global_controls: GlobalControls<O, G>,
    pub voice_controls: [VoiceControls<O, D>; NUMBER_OF_VOICES],
}

struct MidiStream<Rx: RxMidiStream, Tx: TxMidiStream> {
    pub rx: Rx,
    pub tx: Tx,
}

struct VoiceOscFeedback<F: OscFeedback> {
    pub vco_1: F,
    pub vco_2: F,
    pub post_filter: F,
}

pub struct Board<E: ExtMemory> {
    pub ext_memory: E,
}
