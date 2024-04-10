use crate::bounded::Norm;

pub trait Slider {
    fn get(&self) -> Norm;
}

struct Sliders<'a> {
    vco_tune_1: &'a mut dyn Slider,
    vco_tune_2: &'a mut dyn Slider,
    vco_wave_1: &'a mut dyn Slider,
    vco_wave_2: &'a mut dyn Slider,
    vco_mix: &'a mut dyn Slider,
    vco_mod_1: &'a mut dyn Slider,
    vco_mod_2: &'a mut dyn Slider,
    vco_detune: &'a mut dyn Slider,
    vco_glide: &'a mut dyn Slider,
    filter_cutoff: &'a mut dyn Slider,
    filter_reson: &'a mut dyn Slider,
    filter_env: &'a mut dyn Slider,
    filter_lfo: &'a mut dyn Slider,
    filter_bass: &'a mut dyn Slider,
    filter_fm: &'a mut dyn Slider,
    master_spread: &'a mut dyn Slider,
    bbd_amount: &'a mut dyn Slider,
    bbd_morph: &'a mut dyn Slider,
    delay_mix: &'a mut dyn Slider,
    delay_time: &'a mut dyn Slider,
    delay_feed: &'a mut dyn Slider,
    poly_lfo_fade: &'a mut dyn Slider,
    poly_lfo_rate: &'a mut dyn Slider,
    poly_lfo_spread: &'a mut dyn Slider,
    lfo_b_rate: &'a mut dyn Slider,
    lfo_b_amount: &'a mut dyn Slider,

    vcf_attack: &'a mut dyn Slider,
    vcf_decay: &'a mut dyn Slider,
    vcf_sustain: &'a mut dyn Slider,
    vcf_release: &'a mut dyn Slider,

    vca_attack: &'a mut dyn Slider,
    vca_decay: &'a mut dyn Slider,
    vca_sustain: &'a mut dyn Slider,
    vca_release: &'a mut dyn Slider,
}
