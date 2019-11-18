use amethyst::ecs::prelude::{Component, VecStorage};

pub struct Player{
    pub width: usize,
    pub height: usize,
}

impl Player{
    pub fn new(width: usize, height: usize) -> Player {
        Player {
            width,
            height,
        }
    }
}

impl Component for Player{
    type Storage = VecStorage<Self>;
}