use fieldset::*;

use crate::{
    board::{Button, Buttons, Slider, Sliders},
    bounded::Norm,
};

#[derive(FieldSet)]
pub struct UserInputs {
    lfo_1_rate: Norm,
    lfo_1_fade: Norm,
    lfo_1_vco_amount: Norm,
    lfo_1_vcf_amount: Norm,
    lfo_2_rate: Norm,
    lfo_2_xmod: Norm,
    lfo_2_morph_amount: Norm,
    lfo_2_vco_1_pw_amount: Norm,
    vco_sub_noise: Norm,
    vco_mix: Norm,
    vco_morph: Norm,
    vco_2_tune: Norm,
    vco_detune: Norm,
    vco_glide: Norm,
    vco_pw: Norm,
    vco_fm: Norm,
    lpf_cut: Norm,
    lpf_cut_eg_amount: Norm,
    lpf_reson: Norm,
    lpf_ffm: Norm,
    hpf_cut: Norm,
    hpf_reson: Norm,
    master_volume: Norm,
    spread: Norm,
    s_a: Norm,
    s_d: Norm,
    s_s: Norm,
    s_r: Norm,
    s_1: Norm,
    s_2: Norm,
    s_3: Norm,
    s_4: Norm,
    s_5: Norm,
    lfo_1_wave: bool,
    lfo_1_vco_target: bool,
    lfo_2_wave: bool,
    lfo_2_sync_mode: bool,
    vco_sync: bool,
    vco_fm_eg: bool,
    lpf_poles: bool,
    lpf_track: bool,
    lpf_ffm_noise_source: bool,
    eg_mode: bool,
    play: bool,
    stop: bool,
    rec: bool,
    menu: bool,
    save: bool,
    dist: bool,
    modw: bool,
    delay: bool,
    rev: bool,
    seq: bool,
}

pub struct UserInputsService<S: Slider, B: Button> {
    sliders: Sliders<S>,
    buttons: Buttons<B>,
    data: UserInputs,
    fieldset: UserInputsOptFieldSet,
}

pub type UserInputsChangedEvent = UserInputsFieldType;

impl<S: Slider, B: Button> UserInputsService<S, B> {
    pub fn new(sliders: Sliders<S>, buttons: Buttons<B>) -> Self {
        Self {
            sliders,
            buttons,
            data: UserInputs {
                lfo_1_rate: Norm::new(0.).unwrap(),
                lfo_1_fade: Norm::new(0.).unwrap(),
                lfo_1_vco_amount: Norm::new(0.).unwrap(),
                lfo_1_vcf_amount: Norm::new(0.).unwrap(),
                lfo_2_rate: Norm::new(0.).unwrap(),
                lfo_2_xmod: Norm::new(0.).unwrap(),
                lfo_2_morph_amount: Norm::new(0.).unwrap(),
                lfo_2_vco_1_pw_amount: Norm::new(0.).unwrap(),
                vco_sub_noise: Norm::new(0.).unwrap(),
                vco_mix: Norm::new(0.).unwrap(),
                vco_morph: Norm::new(0.).unwrap(),
                vco_2_tune: Norm::new(0.).unwrap(),
                vco_detune: Norm::new(0.).unwrap(),
                vco_glide: Norm::new(0.).unwrap(),
                vco_pw: Norm::new(0.).unwrap(),
                vco_fm: Norm::new(0.).unwrap(),
                lpf_cut: Norm::new(0.).unwrap(),
                lpf_cut_eg_amount: Norm::new(0.).unwrap(),
                lpf_reson: Norm::new(0.).unwrap(),
                lpf_ffm: Norm::new(0.).unwrap(),
                hpf_cut: Norm::new(0.).unwrap(),
                hpf_reson: Norm::new(0.).unwrap(),
                master_volume: Norm::new(0.).unwrap(),
                spread: Norm::new(0.).unwrap(),
                s_a: Norm::new(0.).unwrap(),
                s_d: Norm::new(0.).unwrap(),
                s_s: Norm::new(0.).unwrap(),
                s_r: Norm::new(0.).unwrap(),
                s_1: Norm::new(0.).unwrap(),
                s_2: Norm::new(0.).unwrap(),
                s_3: Norm::new(0.).unwrap(),
                s_4: Norm::new(0.).unwrap(),
                s_5: Norm::new(0.).unwrap(),
                lfo_1_wave: false,
                lfo_1_vco_target: false,
                lfo_2_wave: false,
                lfo_2_sync_mode: false,
                vco_sync: false,
                vco_fm_eg: false,
                lpf_poles: false,
                lpf_track: false,
                lpf_ffm_noise_source: false,
                eg_mode: false,
                play: false,
                stop: false,
                rec: false,
                menu: false,
                save: false,
                dist: false,
                modw: false,
                delay: false,
                rev: false,
                seq: false,
            },
            fieldset: Default::default(),
        }
    }

    pub fn poll(&mut self) -> impl Iterator<Item = UserInputsChangedEvent> {
        if self.sliders.lfo_1_rate.get() != self.data.lfo_1_rate {
            self.fieldset
                .lfo_1_rate()
                .set(self.sliders.lfo_1_rate.get());
        }
        if self.sliders.lfo_1_fade.get() != self.data.lfo_1_fade {
            self.fieldset
                .lfo_1_fade()
                .set(self.sliders.lfo_1_fade.get());
        }
        if self.sliders.lfo_1_vco_amount.get() != self.data.lfo_1_vco_amount {
            self.fieldset
                .lfo_1_vco_amount()
                .set(self.sliders.lfo_1_vco_amount.get());
        }
        if self.sliders.lfo_1_vcf_amount.get() != self.data.lfo_1_vcf_amount {
            self.fieldset
                .lfo_1_vcf_amount()
                .set(self.sliders.lfo_1_vcf_amount.get());
        }
        if self.sliders.lfo_2_rate.get() != self.data.lfo_2_rate {
            self.fieldset
                .lfo_2_rate()
                .set(self.sliders.lfo_2_rate.get());
        }
        if self.sliders.lfo_2_xmod.get() != self.data.lfo_2_xmod {
            self.fieldset
                .lfo_2_xmod()
                .set(self.sliders.lfo_2_xmod.get());
        }
        if self.sliders.lfo_2_morph_amount.get() != self.data.lfo_2_morph_amount {
            self.fieldset
                .lfo_2_morph_amount()
                .set(self.sliders.lfo_2_morph_amount.get());
        }
        if self.sliders.lfo_2_vco_1_pw_amount.get() != self.data.lfo_2_vco_1_pw_amount {
            self.fieldset
                .lfo_2_vco_1_pw_amount()
                .set(self.sliders.lfo_2_vco_1_pw_amount.get());
        }
        if self.sliders.vco_sub_noise.get() != self.data.vco_sub_noise {
            self.fieldset
                .vco_sub_noise()
                .set(self.sliders.vco_sub_noise.get());
        }
        if self.sliders.vco_mix.get() != self.data.vco_mix {
            self.fieldset.vco_mix().set(self.sliders.vco_mix.get());
        }
        if self.sliders.vco_morph.get() != self.data.vco_morph {
            self.fieldset.vco_morph().set(self.sliders.vco_morph.get());
        }
        if self.sliders.vco_2_tune.get() != self.data.vco_2_tune {
            self.fieldset
                .vco_2_tune()
                .set(self.sliders.vco_2_tune.get());
        }
        if self.sliders.vco_detune.get() != self.data.vco_detune {
            self.fieldset
                .vco_detune()
                .set(self.sliders.vco_detune.get());
        }
        if self.sliders.vco_glide.get() != self.data.vco_glide {
            self.fieldset.vco_glide().set(self.sliders.vco_glide.get());
        }
        if self.sliders.vco_pw.get() != self.data.vco_pw {
            self.fieldset.vco_pw().set(self.sliders.vco_pw.get());
        }
        if self.sliders.vco_fm.get() != self.data.vco_fm {
            self.fieldset.vco_fm().set(self.sliders.vco_fm.get());
        }
        if self.sliders.lpf_cut.get() != self.data.lpf_cut {
            self.fieldset.lpf_cut().set(self.sliders.lpf_cut.get());
        }
        if self.sliders.lpf_cut_eg_amount.get() != self.data.lpf_cut_eg_amount {
            self.fieldset
                .lpf_cut_eg_amount()
                .set(self.sliders.lpf_cut_eg_amount.get());
        }
        if self.sliders.lpf_reson.get() != self.data.lpf_reson {
            self.fieldset.lpf_reson().set(self.sliders.lpf_reson.get());
        }
        if self.sliders.lpf_ffm.get() != self.data.lpf_ffm {
            self.fieldset.lpf_ffm().set(self.sliders.lpf_ffm.get());
        }
        if self.sliders.hpf_cut.get() != self.data.hpf_cut {
            self.fieldset.hpf_cut().set(self.sliders.hpf_cut.get());
        }
        if self.sliders.hpf_reson.get() != self.data.hpf_reson {
            self.fieldset.hpf_reson().set(self.sliders.hpf_reson.get());
        }
        if self.sliders.master_volume.get() != self.data.master_volume {
            self.fieldset
                .master_volume()
                .set(self.sliders.master_volume.get());
        }
        if self.sliders.spread.get() != self.data.spread {
            self.fieldset.spread().set(self.sliders.spread.get());
        }
        if self.sliders.s_a.get() != self.data.s_a {
            self.fieldset.s_a().set(self.sliders.s_a.get());
        }
        if self.sliders.s_d.get() != self.data.s_d {
            self.fieldset.s_d().set(self.sliders.s_d.get());
        }
        if self.sliders.s_s.get() != self.data.s_s {
            self.fieldset.s_s().set(self.sliders.s_s.get());
        }
        if self.sliders.s_r.get() != self.data.s_r {
            self.fieldset.s_r().set(self.sliders.s_r.get());
        }
        if self.sliders.s_1.get() != self.data.s_1 {
            self.fieldset.s_1().set(self.sliders.s_1.get());
        }
        if self.sliders.s_2.get() != self.data.s_2 {
            self.fieldset.s_2().set(self.sliders.s_2.get());
        }
        if self.sliders.s_3.get() != self.data.s_3 {
            self.fieldset.s_3().set(self.sliders.s_3.get());
        }
        if self.sliders.s_4.get() != self.data.s_4 {
            self.fieldset.s_4().set(self.sliders.s_4.get());
        }
        if self.sliders.s_5.get() != self.data.s_5 {
            self.fieldset.s_5().set(self.sliders.s_5.get());
        }
        if self.buttons.lfo_1_wave.get() != self.data.lfo_1_wave {
            self.fieldset
                .lfo_1_wave()
                .set(self.buttons.lfo_1_wave.get());
        }
        if self.buttons.lfo_1_vco_target.get() != self.data.lfo_1_vco_target {
            self.fieldset
                .lfo_1_vco_target()
                .set(self.buttons.lfo_1_vco_target.get());
        }
        if self.buttons.lfo_2_wave.get() != self.data.lfo_2_wave {
            self.fieldset
                .lfo_2_wave()
                .set(self.buttons.lfo_2_wave.get());
        }
        if self.buttons.lfo_2_sync_mode.get() != self.data.lfo_2_sync_mode {
            self.fieldset
                .lfo_2_sync_mode()
                .set(self.buttons.lfo_2_sync_mode.get());
        }
        if self.buttons.vco_sync.get() != self.data.vco_sync {
            self.fieldset.vco_sync().set(self.buttons.vco_sync.get());
        }
        if self.buttons.vco_fm_eg.get() != self.data.vco_fm_eg {
            self.fieldset.vco_fm_eg().set(self.buttons.vco_fm_eg.get());
        }
        if self.buttons.lpf_poles.get() != self.data.lpf_poles {
            self.fieldset.lpf_poles().set(self.buttons.lpf_poles.get());
        }
        if self.buttons.lpf_track.get() != self.data.lpf_track {
            self.fieldset.lpf_track().set(self.buttons.lpf_track.get());
        }
        if self.buttons.lpf_ffm_noise_source.get() != self.data.lpf_ffm_noise_source {
            self.fieldset
                .lpf_ffm_noise_source()
                .set(self.buttons.lpf_ffm_noise_source.get());
        }
        if self.buttons.eg_mode.get() != self.data.eg_mode {
            self.fieldset.eg_mode().set(self.buttons.eg_mode.get());
        }
        if self.buttons.play.get() != self.data.play {
            self.fieldset.play().set(self.buttons.play.get());
        }
        if self.buttons.stop.get() != self.data.stop {
            self.fieldset.stop().set(self.buttons.stop.get());
        }
        if self.buttons.rec.get() != self.data.rec {
            self.fieldset.rec().set(self.buttons.rec.get());
        }
        if self.buttons.menu.get() != self.data.menu {
            self.fieldset.menu().set(self.buttons.menu.get());
        }
        if self.buttons.save.get() != self.data.save {
            self.fieldset.save().set(self.buttons.save.get());
        }
        if self.buttons.dist.get() != self.data.dist {
            self.fieldset.dist().set(self.buttons.dist.get());
        }
        if self.buttons.modw.get() != self.data.modw {
            self.fieldset.modw().set(self.buttons.modw.get());
        }
        if self.buttons.delay.get() != self.data.delay {
            self.fieldset.delay().set(self.buttons.delay.get());
        }
        if self.buttons.rev.get() != self.data.rev {
            self.fieldset.rev().set(self.buttons.rev.get());
        }
        if self.buttons.seq.get() != self.data.seq {
            self.fieldset.seq().set(self.buttons.seq.get());
        }
        core::mem::replace(&mut self.fieldset, Default::default()).into_iter()
    }
}
