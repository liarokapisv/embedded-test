#![no_std]
#![allow(dead_code)]
#![feature(effects)]

pub use core::marker::PhantomData;
pub use fieldset_macro::*;

pub trait FieldSetter<T> {
    fn set(&mut self, value: T);
}

impl<T> FieldSetter<T> for &mut Option<T> {
    fn set(&mut self, value: T) {
        **self = Some(value);
    }
}

pub struct BitFieldLeafSetter<'a, V, T, F>(
    pub &'a mut [u32],
    pub &'a mut [T],
    pub &'a mut usize,
    pub usize,
    pub F,
    pub PhantomData<V>,
);

pub struct BitFieldSetters<'a, T, F>(pub &'a mut [u32], pub &'a mut [T], pub &'a mut usize, pub F);

impl<'a, V, T, F: Fn(V) -> T> FieldSetter<V> for BitFieldLeafSetter<'a, V, T, F> {
    fn set(&mut self, value: V) {
        if self.0[self.3 / 32] & (1 << (self.3 % 32)) == 0 {
            self.0[self.3 / 32] |= 1 << (self.3 % 32);
            self.1[*self.2] = self.4(value);
            *self.2 += 1;
        }
    }
}

pub struct PerfFieldLeafSetter<'a, V, T, F>(
    pub &'a mut [u16],
    pub &'a mut [T],
    pub &'a mut usize,
    pub usize,
    pub F,
    pub PhantomData<V>,
);

pub struct PerfFieldSetters<'a, T, F>(pub &'a mut [u16], pub &'a mut [T], pub &'a mut usize, pub F);

impl<'a, V, T, F: Fn(V) -> T> FieldSetter<V> for PerfFieldLeafSetter<'a, V, T, F> {
    fn set(&mut self, value: V) {
        if self.0[self.3] == 0 {
            self.0[self.3] = *self.2 as u16 + 1;
            self.1[*self.2] = self.4(value);
            *self.2 += 1;
        } else {
            self.1[self.0[self.3] as usize - 1] = self.4(value);
        }
    }
}
