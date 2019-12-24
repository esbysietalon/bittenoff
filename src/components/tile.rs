use amethyst::ecs::prelude::{Component, VecStorage};

pub struct Tile{
    index: usize,
}

impl Tile {
    pub fn new(index: usize) -> Tile{
        Tile{
            index,
        }
    }
    pub fn index(&self) -> usize {
        self.index
    }
}

impl Component for Tile {
    type Storage = VecStorage<Self>;
}