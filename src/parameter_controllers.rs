#[allow(dead_code)]

use crate::bounded::Bounded;

struct PrevInput<T>(Option<T>);

trait ParameterController<T> {
    fn poll(&mut self, current_value: T, new_input: T) -> Option<T>;
    fn sync(&mut self, prev_input: Option<T>);
    fn reset(&mut self) {
        self.sync(None)
    }
}

impl<T> PrevInput<T> {
    pub fn poll(&mut self, new_input: T) -> Option<T>
    where
        T: Copy,
    {
        let prev_input = self.0;
        self.0 = Some(new_input);
        prev_input
    }

    pub fn sync(&mut self, value: Option<T>) {
        self.0 = value;
    }
}

struct JumpController<T> {
    prev_input: PrevInput<T>,
}

impl<T> JumpController<T> {
    fn poll(&mut self, new_input: T) -> Option<T>
    where
        T: PartialEq + Copy,
    {
        if self.prev_input.poll(new_input)? != new_input {
            Some(new_input)
        } else {
            None
        }
    }
}

impl<T: PartialEq + Copy> ParameterController<T> for JumpController<T> {
    fn poll(&mut self, _current_value: T, new_input: T) -> Option<T> {
        self.poll(new_input)
    }

    fn sync(&mut self, prev_input: Option<T>) {
        self.prev_input.sync(prev_input);
    }
}

struct CatchController<T> {
    prev_input: PrevInput<T>,
}

impl<T: PartialOrd + Copy> ParameterController<T> for CatchController<T> {
    fn poll(&mut self, current_value: T, new_input: T) -> Option<T> {
        let prev_input = self.prev_input.poll(new_input)?;

        if new_input == prev_input
            || new_input > current_value && current_value > prev_input
            || new_input < current_value && current_value < prev_input
        {
            Some(new_input)
        } else {
            None
        }
    }

    fn sync(&mut self, prev_input: Option<T>) {
        self.prev_input.sync(prev_input);
    }
}

struct ScaleController<T> {
    prev_input: PrevInput<T>,
}

fn map(x: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    to_min + (x - from_min) / (from_max - from_min) * (to_max - to_min)
}

impl<const MIN: i32, const MAX: i32> ParameterController<Bounded<f32, MIN, MAX>>
    for ScaleController<Bounded<f32, MIN, MAX>>
{
    fn poll(
        &mut self,
        current_value: Bounded<f32, MIN, MAX>,
        new_input: Bounded<f32, MIN, MAX>,
    ) -> Option<Bounded<f32, MIN, MAX>> {
        let prev_input = self.prev_input.poll(new_input)?.get();
        let new_input = new_input.get();
        let current_value = current_value.get();

        if new_input < prev_input {
            return Some(
                Bounded::new(
                    map(new_input, MIN as f32, prev_input, MIN as f32, current_value)
                        .clamp(MIN as f32, MAX as f32),
                )
                .unwrap(),
            );
        }

        if new_input > prev_input {
            return Some(
                Bounded::new(
                    map(new_input, prev_input, MAX as f32, current_value, MAX as f32)
                        .clamp(MIN as f32, MAX as f32),
                )
                .unwrap(),
            );
        }

        None
    }

    fn sync(&mut self, prev_input: Option<Bounded<f32, MIN, MAX>>) {
        self.prev_input.sync(prev_input);
    }
}
