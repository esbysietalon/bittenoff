use amethyst::ecs::prelude::{Component, VecStorage};

pub struct Counter{
    fps: u32,
    time: f32,
}

impl Counter{
    pub fn new() -> Counter {
        Counter {
            fps: 0,
            time: 0.0,
        }
    }    

    pub fn tick(&mut self, added_time: f32){
        self.time += added_time;
        self.fps += 1;
        if self.time > 1.0 {
            println!("FPS: {}", self.fps);
            self.time = 0.0;
            self.fps = 0;
        }
    }
}

impl Component for Counter{
    type Storage = VecStorage<Self>;
}