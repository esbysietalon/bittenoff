use amethyst::ecs::prelude::{Component, VecStorage};

pub struct Plant{
    fruiting: bool,
    fruit_rate: f32,
    fruit_progress: f32,
}

impl Plant {
    pub fn new(fruiting: bool, fruit_rate: f32, fruit_progress: f32) -> Plant{
        Plant{
            fruiting,
            fruit_rate,
            fruit_progress,
        }
    }
    pub fn get_fruiting(&self) -> bool {
        self.fruiting
    }
    pub fn get_fruit_rate(&self) -> f32 {
        self.fruit_rate
    }
    pub fn get_fruit_progress(&self) -> f32 {
        self.fruit_progress
    }
    pub fn mut_fruit_progress(&mut self, x: f32) {
        self.fruit_progress += x;
    }
}

impl Component for Plant {
    type Storage = VecStorage<Self>;
}