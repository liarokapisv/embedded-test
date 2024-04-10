use num::cast::AsPrimitive;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Bounded<T, const Min: i32, const Max: i32>(T);

impl<T, const Min: i32, const Max: i32> Bounded<T, Min, Max> {
    pub unsafe fn new_unchecked(value: T) -> Self {
        Bounded(value)
    }

    pub fn new(value: T) -> Option<Self>
    where
        T: PartialOrd + Copy + 'static,
        i32: AsPrimitive<T>,
    {
        if value < Min.as_() || value > Max.as_() {
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

pub type BoundedFloat<const Min: i32, const Max: i32> = Bounded<f32, Min, Max>;

pub type Norm = BoundedFloat<0, 1>;
pub type SNorm = BoundedFloat<-1, 1>;
