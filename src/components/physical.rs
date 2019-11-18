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
    pub fn move_tile(&mut self, mv: &str) {
        if mv == "l" {
            self.position.0 -= 1;
        } else if mv == "r" {
            self.position.0 += 1;
        } else if mv == "u" {
            self.position.1 += 1;
        } else if mv == "d" {
            self.position.1 -= 1;
        }
    }
         
}

impl Component for Physical{
    type Storage = VecStorage<Self>;
}