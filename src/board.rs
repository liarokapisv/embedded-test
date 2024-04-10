use core::future::Future;
use core::pin::Pin;
use core::ptr::DynMetadata;
use core::ptr::Pointee;
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
    lfo_1_rate: T,
    lfo_1_fade: T,
    lfo_1_vco_amount: T,
    lfo_1_vcf_amount: T,
    lfo_2_rate: T,
    lfo_2_xmod: T,
    lfo_2_morph_amount: T,
    lfo_2_vco_1_pw_amount: T,
    vco_sub_noise: T,
    vco_mix: T,
    vco_morph: T,
    vco_2_tune: T,
    vco_detune: T,
    vco_glide: T,
    vco_pw: T,
    vco_fm: T,
    lpf_cut: T,
    lpf_cut_eg_amount: T,
    lpf_reson: T,
    lpf_ffm: T,
    hpf_cut: T,
    hpf_reson: T,
    master_volume: T,
    spread: T,
    s_a: T,
    s_d: T,
    s_s: T,
    s_r: T,
    s_1: T,
    s_2: T,
    s_3: T,
    s_4: T,
    s_5: T,
}

struct ButtonLike<T> {
    lfo_1_wave: T,
    lfo_1_vco_target: T,
    lfo_2_wave: T,
    lfo_2_sync_mode: T,
    vco_sync: T,
    vco_fm_eg: T,
    lpf_poles: T,
    lpf_track: T,
    lpf_ffm_noise_source: T,
    eg_mode: T,

    play: T,
    stop: T,
    rec: T,
    menu: T,
    save: T,
    dist: T,
    modw: T,
    delay: T,
    rev: T,
    seq: T,
}

type Buttons<T: Button> = ButtonLike<T>;
type Leds<T: Led> = ButtonLike<T>;

struct GlobalControls<O: OpaqueControl, G: Gpio> {
    vco_1_level: O,
    vco_sub_noise: O,
    vco_2_pw: O,
    vco_2_level: O,
    vco_sync: O,
    lpf_reson: O,
    lpf_ffm_noise_source: G,
    lpf_ffm: O,
    lpf_poles: G,
}

struct VoiceControls<O: OpaqueControl, D: Dac> {
    vco_1_tune_main: D,
    vco_1_tune_offset: D,
    vco_2_tune_main: D,
    vco_2_tune_offset: D,
    vco_1_pw: O,
    vco_fm: O,
    vco_morph: O,
    lpf_cut_main: D,
    lpf_cut_offset: D,
    vca_eg: O,
}

struct Controls<O: OpaqueControl, G: Gpio, D: Dac> {
    global_controls: GlobalControls<O, G>,
    voice_controls: [VoiceControls<O, D>; NUMBER_OF_VOICES],
}

struct MidiStream<Rx: RxMidiStream, Tx: TxMidiStream> {
    rx: Rx,
    tx: Tx,
}

struct VoiceOscFeedback<F: OscFeedback> {
    vco_1: F,
    vco_2: F,
    post_filter: F,
}

pub struct Board<E: ExtMemory> {
    pub ext_memory: E,
}
