use amethyst::ecs::prelude::{Component, VecStorage};
use crate::components::Id;

pub struct Hunger{
    capacity: f32,
    rate: f32, //per second
    current: f32,
    to_eat_id: Id,
}

impl Hunger {
    pub fn new(capacity: f32, rate: f32, current: f32) -> Hunger{
        Hunger{
            capacity,
            rate,
            current,
            to_eat_id: Id::nil(),
        }
    }
    pub fn set_meal_id(&mut self, id: Id) {
        self.to_eat_id = id;
    }
    pub fn get_meal_id(&self) -> Id {
        self.to_eat_id
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
    pub fn set_hunger(&mut self, x: f32) {
        self.current = x;
    }
}

impl Component for Hunger {
    type Storage = VecStorage<Self>;
}