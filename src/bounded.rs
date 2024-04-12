use num::cast::AsPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Bounded<T, const MIN: i32, const MAX: i32>(T);

impl<T, const MIN: i32, const MAX: i32> Bounded<T, MIN, MAX> {
    #![allow(dead_code)]
    pub unsafe fn new_unchecked(value: T) -> Self {
        Bounded(value)
    }

    pub fn new(value: T) -> Option<Self>
    where
        T: PartialOrd + Copy + 'static,
        i32: AsPrimitive<T>,
    {
        if value < MIN.as_() || value > MAX.as_() {
            return None;
        }

        Some(Bounded(value))
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        self.0
    }
}

pub type BoundedFloat<const MIN: i32, const MAX: i32> = Bounded<f32, MIN, MAX>;

pub type Norm = BoundedFloat<0, 1>;
pub type SNorm = BoundedFloat<-1, 1>;
