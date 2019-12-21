use amethyst::ecs::prelude::{Component, VecStorage};
use crate::game_state::TILE_SIZE;

#[derive(Clone)]
pub struct Physical{
    position: (usize, usize),
}

impl Physical{
    pub fn new(position: (usize, usize)) -> Physical {
        Physical {
            position,
        }
    }
    pub fn set_tile_position(&mut self, real_position: (f32, f32)) -> (usize, usize) {
        let mut tile_x = 0;
        let mut tile_y = 0;

        tile_x = (real_position.0 as usize) / TILE_SIZE;
        tile_y = (real_position.1 as usize) / TILE_SIZE;

        self.position = (tile_x, tile_y);
        self.position
    }
    pub fn into_tile_position(real_position: (f32, f32)) -> (usize, usize) {
        let mut tile_x = 0;
        let mut tile_y = 0;

        tile_x = (real_position.0 as usize) / TILE_SIZE;
        tile_y = (real_position.1 as usize) / TILE_SIZE;

        (tile_x, tile_y)
    }
    pub fn get_tile_position(&self) -> (usize, usize) {
        self.position
    }
         
}

impl Component for Physical{
    type Storage = VecStorage<Self>;
}