#![no_std]
#![allow(dead_code)]

pub use fieldset_macro::*;

pub trait FieldSetter<T> {
    fn set(&mut self, value: T);
}

impl<T> FieldSetter<T> for &mut Option<T> {
    fn set(&mut self, value: T) {
        **self = Some(value);
    }
}
