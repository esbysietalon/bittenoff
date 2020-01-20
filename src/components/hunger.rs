use amethyst::ecs::prelude::{Component, VecStorage};

pub struct Hunger{
    capacity: f32,
    rate: f32, //per second
    current: f32,
}

impl Hunger {
    pub fn new(capacity: f32, rate: f32) -> Hunger{
        Hunger{
            capacity,
            rate,
            current: capacity,
        }
    }
    pub fn get_hunger(&self) -> f32 {
        self.current
    }
    pub fn get_capacity(&self) -> f32 {
        self.capacity
    }
    pub fn get_rate(&self) -> f32 {
        self.rate
    }
    pub fn mut_hunger(&mut self, x: f32) {
        self.current += x;
        if self.current < 0.0 {
            self.current = 0.0;
        }else if self.current > self.capacity {
            self.current = self.capacity;
        }
    }
}

impl Component for Hunger {
    type Storage = VecStorage<Self>;
}