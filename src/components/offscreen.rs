use amethyst::ecs::prelude::{Component, VecStorage};

pub struct Offscreen {
    offscreen_time: f32,    
}

impl Offscreen {
    pub fn new() -> Offscreen {
        Offscreen {
            offscreen_time: 0.0,
        }
    }
    pub fn tick(&mut self, time: f32) {
        self.offscreen_time += time;
    }
    pub fn time_passed(&self) -> f32 {
        self.offscreen_time
    }
    pub fn reset(&mut self) {
        self.offscreen_time = 0.0;
    }
}

impl Component for Offscreen{
    type Storage = VecStorage<Self>;
}